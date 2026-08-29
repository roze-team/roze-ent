use super::super::*;

pub async fn list_user_groups(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetUserReq,
) -> Result<ListGroupsResp, RozeError> {
    let _ = request_ctx;
    let user = ctx
        .model()
        .user()
        .find_by_id(req.id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("user {} not found", req.id)))?;
    let groups = user
        .query_groups(&ctx.model().membership(), &ctx.model().group())
        .await
        .map_err(model_error)?
        .into_iter()
        .map(group_response)
        .collect();
    Ok(ListGroupsResp { groups })
}
