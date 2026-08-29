use super::super::*;

pub async fn create_user(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: CreateUserReq,
) -> Result<UserResp, RozeError> {
    let _ = request_ctx;
    let user = ctx
        .model()
        .user()
        .create()
        .set_email(req.email)
        .set_name(req.name)
        .save()
        .await
        .map_err(model_error)?;
    Ok(user_response(user))
}
