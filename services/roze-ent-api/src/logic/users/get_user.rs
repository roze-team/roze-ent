use super::super::*;

pub async fn get_user(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetUserReq,
) -> Result<UserResp, RozeError> {
    let _ = request_ctx;
    let user = ctx
        .model()
        .user()
        .find_by_id(req.id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("user {} not found", req.id)))?;
    Ok(user_response(user))
}
