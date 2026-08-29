use super::super::*;

pub async fn create_pet(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: CreatePetReq,
) -> Result<PetResp, RozeError> {
    let _ = request_ctx;
    ctx.model()
        .user()
        .find_by_id_primary(req.owner_id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("user {} not found", req.owner_id)))?;

    let pet = ctx
        .model()
        .pet()
        .create()
        .set_owner_id(req.owner_id)
        .set_name(req.name)
        .set_species(req.species)
        .save()
        .await
        .map_err(model_error)?;
    Ok(pet_response(pet))
}
