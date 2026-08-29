use super::super::*;

pub async fn remove_group_member(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: MembershipPathReq,
) -> Result<DeleteResp, RozeError> {
    let _ = request_ctx;
    let user = ctx
        .model()
        .user()
        .find_by_id_primary(req.user_id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("user {} not found", req.user_id)))?;
    let group = ctx
        .model()
        .group()
        .find_by_id_primary(req.group_id)
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::NotFound(format!("group {} not found", req.group_id)))?;

    let rows_affected = user
        .remove_groups(&group, &ctx.model().membership())
        .await
        .map_err(model_error)?;
    if rows_affected == 0 {
        return Err(RozeError::NotFound(format!(
            "membership for user {} and group {} not found",
            req.user_id, req.group_id
        )));
    }
    Ok(DeleteResp { deleted: true })
}
