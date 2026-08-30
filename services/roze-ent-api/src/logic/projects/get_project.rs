use super::super::*;

pub async fn get_project(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: ProjectPathReq,
) -> Result<ProjectResp, RozeError> {
    let tenant_id = authorized_tenant(&request_ctx, &req.tenant_id)?;
    let project = ctx
        .model()
        .project()
        .find_by_id_for_tenant_id(req.id, tenant_id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("project {} not found", req.id)))?;
    Ok(project_response(project))
}
