use super::super::*;

pub async fn delete_pet(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetPetReq,
) -> Result<DeleteResp, RozeError> {
    let _ = request_ctx;
    let result = ctx
        .model()
        .pet()
        .delete_one(req.id)
        .exec()
        .await
        .map_err(model_error)?;
    if result.rows_affected == 0 {
        return Err(RozeError::NotFound(format!("pet {} not found", req.id)));
    }
    Ok(DeleteResp { deleted: true })
}
