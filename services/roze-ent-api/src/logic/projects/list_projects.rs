use super::super::*;

pub async fn list_projects(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: ListProjectsReq,
) -> Result<ListProjectsResp, RozeError> {
    let tenant_id = authorized_tenant(&request_ctx, &req.tenant_id)?;
    let projects = ctx
        .model()
        .project()
        .query()
        .where_(crate::model::project::tenant_id_eq(tenant_id))
        .order_by_id_asc()
        .limit(100)
        .all()
        .await
        .map_err(model_error)?
        .into_iter()
        .map(project_response)
        .collect();
    Ok(ListProjectsResp { projects })
}
