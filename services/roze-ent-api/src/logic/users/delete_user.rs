use super::super::*;

pub async fn delete_user(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetUserReq,
) -> Result<DeleteResp, RozeError> {
    let _ = request_ctx;
    let result = ctx
        .model()
        .user()
        .delete_one(req.id)
        .exec()
        .await
        .map_err(model_error)?;
    if result.rows_affected == 0 {
        return Err(RozeError::NotFound(format!("user {} not found", req.id)));
    }
    Ok(DeleteResp { deleted: true })
}
