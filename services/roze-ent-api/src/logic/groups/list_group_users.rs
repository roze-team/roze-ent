use super::super::*;

pub async fn list_group_users(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetGroupReq,
) -> Result<ListUsersResp, RozeError> {
    let _ = request_ctx;
    let group = ctx
        .model()
        .group()
        .find_by_id(req.id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("group {} not found", req.id)))?;
    let users = group
        .query_users(&ctx.model().membership(), &ctx.model().user())
        .await
        .map_err(model_error)?
        .into_iter()
        .map(user_response)
        .collect();
    Ok(ListUsersResp { users })
}
