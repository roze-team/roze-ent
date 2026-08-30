use crate::svc::ServiceContext;

#[path = "model/user_activity_view.rs"]
pub mod user_activity_view;

/// Stable application-owned hook for attaching data sources and other resources.
///
/// This file is preserved by `rozectl ... generate --update`.
pub async fn configure_context(ctx: ServiceContext) -> anyhow::Result<ServiceContext> {
    Ok(ctx)
}

/// Registers application-owned workers and background services.
///
/// Every registered service shares Roze's shutdown signal and failure propagation.
/// This file and the hook body are preserved by `rozectl ... generate --update`.
pub fn register_services(
    group: &mut roze_service::ServiceGroup,
    ctx: &ServiceContext,
) -> anyhow::Result<()> {
    let _ = group;
    let _ = ctx;
    Ok(())
}
