use super::super::*;

pub async fn list_users(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: EmptyReq,
) -> Result<ListUsersResp, RozeError> {
    let _ = request_ctx;
    let _ = req;
    let users = ctx
        .model()
        .user()
        .query()
        .order_by_id_asc()
        .limit(100)
        .all()
        .await
        .map_err(model_error)?
        .into_iter()
        .map(user_response)
        .collect();
    Ok(ListUsersResp { users })
}
