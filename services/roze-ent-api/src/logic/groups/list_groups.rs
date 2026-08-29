use super::super::*;

pub async fn list_groups(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: EmptyReq,
) -> Result<ListGroupsResp, RozeError> {
    let _ = request_ctx;
    let _ = req;
    let groups = ctx
        .model()
        .group()
        .query()
        .order_by_id_asc()
        .limit(100)
        .all()
        .await
        .map_err(model_error)?
        .into_iter()
        .map(group_response)
        .collect();
    Ok(ListGroupsResp { groups })
}
