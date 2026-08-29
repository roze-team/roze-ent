use super::super::*;

pub async fn list_user_pets(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetUserReq,
) -> Result<ListPetsResp, RozeError> {
    let _ = request_ctx;
    ctx.model()
        .user()
        .find_by_id(req.id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("user {} not found", req.id)))?;

    let pets = ctx
        .model()
        .pet()
        .query()
        .where_(crate::model::pet::owner_id_eq(req.id))
        .order_by_id_asc()
        .all()
        .await
        .map_err(model_error)?
        .into_iter()
        .map(pet_response)
        .collect();
    Ok(ListPetsResp { pets })
}
