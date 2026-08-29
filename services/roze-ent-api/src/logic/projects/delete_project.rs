use super::super::*;

pub async fn delete_project(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: ProjectPathReq,
) -> Result<DeleteResp, RozeError> {
    let _ = request_ctx;
    let project_repo = ctx.model().project();
    let exists = project_repo
        .query()
        .primary()
        .where_(crate::model::project::id_eq(req.id))
        .where_(crate::model::project::tenant_id_eq(req.tenant_id))
        .exists()
        .await
        .map_err(model_error)?;
    if !exists {
        return Err(RozeError::NotFound(format!("project {} not found", req.id)));
    }

    let result = project_repo
        .soft_delete_by_id(req.id)
        .await
        .map_err(model_error)?;
    if result.rows_affected == 0 {
        return Err(RozeError::FailedPrecondition(
            "project could not be soft deleted".to_string(),
        ));
    }
    Ok(DeleteResp { deleted: true })
}
