use super::super::*;

pub async fn get_pet(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetPetReq,
) -> Result<PetResp, RozeError> {
    let _ = request_ctx;
    let pet = ctx
        .model()
        .pet()
        .find_by_id(req.id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("pet {} not found", req.id)))?;
    Ok(pet_response(pet))
}
