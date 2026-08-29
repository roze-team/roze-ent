use super::super::*;

pub async fn get_group(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: GetGroupReq,
) -> Result<GroupResp, RozeError> {
    let _ = request_ctx;
    let group = ctx
        .model()
        .group()
        .find_by_id(req.id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("group {} not found", req.id)))?;
    Ok(group_response(group))
}
