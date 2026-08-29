use super::super::*;

pub async fn add_group_member(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: MembershipPathReq,
) -> Result<MembershipResp, RozeError> {
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

    let existing = ctx
        .model()
        .membership()
        .query()
        .primary()
        .where_(crate::model::membership::user_id_eq(req.user_id))
        .where_(crate::model::membership::group_id_eq(req.group_id))
        .first()
        .await
        .map_err(model_error)?;
    if existing.is_some() {
        return Err(RozeError::Conflict(format!(
            "user {} is already a member of group {}",
            req.user_id, req.group_id
        )));
    }

    let membership = user
        .add_groups(&group, &ctx.model().membership())
        .await
        .map_err(model_error)?;
    Ok(membership_response(membership))
}
