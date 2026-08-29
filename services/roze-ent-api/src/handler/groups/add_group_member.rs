use super::super::*;

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct AddGroupMemberMembershipPathReqPath {
    #[validate(range(min = 1))]
    group_id: i64,
    #[validate(range(min = 1))]
    user_id: i64,
}

pub(crate) async fn add_group_member(
    State(ctx): State<ServiceContext>,
    Extension(request_ctx): Extension<Context>,
    Path(path): Path<AddGroupMemberMembershipPathReqPath>,
    headers: HeaderMap,
    client_ip: Option<roze_http::client_ip::ClientIp>,
) -> Result<ApiResponse<MembershipResp>, RozeError> {
    let request_ctx = match client_ip {
        Some(client_ip) => request_ctx.with_metadata("client_ip", client_ip.to_string()),
        None => request_ctx,
    };
    roze_middleware::enforce_route_rate_limit(
        ctx.rate_limiter.as_ref(),
        ctx.config.name.as_str(),
        "add_group_member",
        "POST",
        &request_ctx,
        client_ip,
        &headers,
        Some(&ctx.config.governance),
    )
    .await?;
    let (request_ctx, route_guard) = roze_middleware::begin_route(
        ctx.config.name.clone(),
        "add_group_member",
        "POST",
        request_ctx,
        Some(&ctx.config.governance),
    )?;
    if let Err(message) =
        roze_validation::validate_or_message_i18n(&path, roze_error::current_locale().as_deref())
    {
        let err = RozeError::BadRequest(message);
        roze_middleware::finish_route(route_guard, false, err.code().to_string());
        return Err(err);
    }
    let req = MembershipPathReq {
        group_id: path.group_id,
        user_id: path.user_id,
    };
    let logic_request_id = request_ctx.request_id();
    let logic_trace_id = request_ctx.trace_id();
    tracing::info!(event = roze_log::events::APPLICATION_LOGIC_STARTED, protocol = "rest", service = %ctx.config.name, operation = "add_group_member", response_kind = "json", request_id = %logic_request_id, trace_id = %logic_trace_id, "REST application logic started");
    let logic_started = std::time::Instant::now();
    let timeout_enabled = ctx
        .config
        .rest
        .as_ref()
        .is_none_or(|rest| rest.middlewares.timeout);
    let timeout = timeout_enabled
        .then(|| request_ctx.remaining_timeout())
        .flatten();
    let logic = crate::logic::add_group_member(ctx.clone(), request_ctx, req);
    let result = match timeout {
        Some(timeout) => match tokio::time::timeout(timeout, logic).await {
            Ok(result) => result,
            Err(_) => Err(RozeError::Internal("request timeout".to_string())),
        },
        None => logic.await,
    };
    match &result {
        Ok(_) => {
            tracing::info!(event = roze_log::events::APPLICATION_LOGIC_COMPLETED, protocol = "rest", service = %ctx.config.name, operation = "add_group_member", response_kind = "json", elapsed_ms = logic_started.elapsed().as_millis(), request_id = %logic_request_id, trace_id = %logic_trace_id, "REST application logic completed")
        }
        Err(error) => {
            tracing::error!(event = roze_log::events::APPLICATION_LOGIC_FAILED, protocol = "rest", service = %ctx.config.name, operation = "add_group_member", response_kind = "json", elapsed_ms = logic_started.elapsed().as_millis(), code = error.code(), error_kind = error.kind(), request_id = %logic_request_id, trace_id = %logic_trace_id, "REST application logic failed")
        }
    }
    match result {
        Ok(resp) => {
            roze_middleware::finish_route(route_guard, true, "200");
            Ok(ApiResponse::ok(resp))
        }
        Err(mut err) => {
            err = roze_middleware::apply_fallback(
                ctx.config.name.as_str(),
                err,
                roze_middleware::route_fallback(Some(&ctx.config.governance), "add_group_member"),
            );
            roze_middleware::finish_route(route_guard, false, err.code().to_string());
            Err(err)
        }
    }
}
