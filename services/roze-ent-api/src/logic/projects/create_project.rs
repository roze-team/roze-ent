use super::super::*;

pub async fn create_project(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: CreateProjectReq,
) -> Result<ProjectResp, RozeError> {
    let tenant_id = authorized_tenant(&request_ctx, &req.tenant_id)?;
    let existing = ctx
        .model()
        .project()
        .query()
        .primary()
        .with_deleted()
        .where_(crate::model::project::tenant_id_eq(tenant_id.clone()))
        .where_(crate::model::project::name_eq(req.name.clone()))
        .first()
        .await
        .map_err(model_error)?;
    if existing.is_some() {
        return Err(RozeError::Conflict(
            "project name already exists for tenant".to_string(),
        ));
    }

    let project = ctx
        .model()
        .project()
        .create()
        .set_tenant_id(tenant_id)
        .set_name(req.name)
        .set_description(req.description)
        .save()
        .await
        .map_err(model_error)?;
    Ok(project_response(project))
}
