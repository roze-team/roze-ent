use super::super::*;

pub async fn update_project(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: UpdateProjectReq,
) -> Result<ProjectResp, RozeError> {
    let _ = request_ctx;
    let next_version = req
        .expected_version
        .checked_add(1)
        .ok_or_else(|| RozeError::BadRequest("project version overflow".to_string()))?;
    let project_repo = ctx.model().project();
    let result = project_repo
        .update_where()
        .where_(crate::model::project::id_eq(req.id))
        .where_(crate::model::project::tenant_id_eq(req.tenant_id.clone()))
        .where_(crate::model::project::version_eq(req.expected_version))
        .set_name(req.name)
        .set_description(req.description)
        .set_version(next_version)
        .execute()
        .await
        .map_err(model_error)?;

    if result.rows_affected == 0 {
        let exists = project_repo
            .query()
            .primary()
            .where_(crate::model::project::id_eq(req.id))
            .where_(crate::model::project::tenant_id_eq(req.tenant_id.clone()))
            .exists()
            .await
            .map_err(model_error)?;
        if exists {
            return Err(RozeError::FailedPrecondition(
                "project version changed; reload and retry".to_string(),
            ));
        }
        return Err(RozeError::NotFound(format!("project {} not found", req.id)));
    }

    let project = project_repo
        .query()
        .primary()
        .where_(crate::model::project::id_eq(req.id))
        .where_(crate::model::project::tenant_id_eq(req.tenant_id))
        .first()
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::Internal("updated project disappeared".to_string()))?;
    Ok(project_response(project))
}
