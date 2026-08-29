use super::super::*;

pub async fn update_group_member(
    ctx: ServiceContext,
    request_ctx: roze_context::Context,
    req: UpdateMembershipRoleReq,
) -> Result<MembershipResp, RozeError> {
    let _ = request_ctx;
    let membership_repo = ctx.model().membership();
    let result = membership_repo
        .update_where()
        .where_(crate::model::membership::user_id_eq(req.user_id))
        .where_(crate::model::membership::group_id_eq(req.group_id))
        .where_(crate::model::membership::role_eq(req.expected_role))
        .set_role(req.role)
        .execute()
        .await
        .map_err(model_error)?;

    if result.rows_affected == 0 {
        let exists = membership_repo
            .query()
            .primary()
            .where_(crate::model::membership::user_id_eq(req.user_id))
            .where_(crate::model::membership::group_id_eq(req.group_id))
            .exists()
            .await
            .map_err(model_error)?;
        if exists {
            return Err(RozeError::FailedPrecondition(
                "membership role changed; reload and retry".to_string(),
            ));
        }
        return Err(RozeError::NotFound(format!(
            "membership for user {} and group {} not found",
            req.user_id, req.group_id
        )));
    }

    let membership = membership_repo
        .query()
        .primary()
        .where_(crate::model::membership::user_id_eq(req.user_id))
        .where_(crate::model::membership::group_id_eq(req.group_id))
        .first()
        .await
        .map_err(model_error)?
        .ok_or_else(|| RozeError::Internal("updated membership disappeared".to_string()))?;
    Ok(membership_response(membership))
}
