use super::super::*;

pub async fn create_group(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: CreateGroupReq,
) -> Result<GroupResp, RozeError> {
    let _ = request_ctx;
    let group = ctx
        .model()
        .group()
        .create()
        .set_name(req.name)
        .set_description(req.description)
        .save()
        .await
        .map_err(model_error)?;
    Ok(group_response(group))
}
