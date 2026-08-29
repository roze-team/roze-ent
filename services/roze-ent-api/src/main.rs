mod application;
mod config;
pub use config::application_config;
mod handler;
mod logic;
mod middleware;
mod model;
mod openapi;
mod route;
mod svc;
mod types;

use roze_http::rest::{RestServer, RestService};
use roze_service::ServiceGroup;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load(config_path())?;
    let _tracing_guard = roze_log::init_tracing_with_config(&config)?;
    let mut rest = config
        .rest
        .clone()
        .ok_or_else(|| anyhow::anyhow!("missing rest config"))?;
    for route in route::WEBSOCKET_PUBLIC_ROUTES {
        if !rest
            .middlewares
            .auth_public_routes
            .iter()
            .any(|configured| configured.trim().eq_ignore_ascii_case(route))
        {
            rest.middlewares
                .auth_public_routes
                .push((*route).to_string());
        }
    }
    tracing::info!(
        event = roze_log::events::SERVICE_CONFIG_LOADED,
        service = %config.name,
        protocol = "rest",
        listen_addr = %rest.addr,
        register = rest.register,
        "service configuration loaded"
    );
    let (mut registration, registry_health) = if rest.register {
        let registry = roze_rpc::registry::build_service_registry(&config)?
            .ok_or_else(|| anyhow::anyhow!("missing registry config"))?;
        let registration = roze_rpc::rpc::ServiceRegistrationGuard::start(
            registry.clone(),
            config.name.clone(),
            rest.addr,
        )
        .await?;
        tracing::info!(event = roze_log::events::SERVICE_REGISTRY_REGISTERED, service = %config.name, protocol = "rest", addr = %rest.addr, "service registered");
        (Some(registration), Some(registry))
    } else {
        (None, None)
    };
    let service_name = config.name.clone();
    let auth_config = config.auth.clone();
    let mut group = ServiceGroup::new();
    let service_shutdown = group.shutdown_listener();
    let ctx = model::configure_context(svc::ServiceContext::new(config).await?).await?;
    let ctx = application::configure_context(ctx).await?;
    tracing::info!(event = roze_log::events::SERVICE_CONTEXT_INITIALIZED, service = %service_name, protocol = "rest", "service context initialized");
    application::register_services(&mut group, &ctx)?;
    let health = ctx.health.clone();
    if let Some(registry) = registry_health {
        let registry_service = service_name.clone();
        health.register_dependency("registry", move || {
            let registry = registry.clone();
            let service = registry_service.clone();
            async move { registry.discover(&service).await.map(|_| ()) }
        });
    }
    if !rest.middlewares.trusted_proxy_cidrs.is_empty() && !rest.connect_info {
        anyhow::bail!("rest.connect_info must be enabled when trusted_proxy_cidrs are configured");
    }
    let middleware_config = roze_middleware::CommonMiddlewareConfig::try_from_service(
        &rest.middlewares,
        auth_config.as_ref(),
    )?;
    tracing::info!(
        event = "rest.middleware.resolved",
        protocol = "rest",
        request_context = middleware_config.request_context,
        request_tracing = middleware_config.tracing,
        auth_enabled = middleware_config.auth.is_some(),
        cors_enabled = middleware_config.cors,
        timeout_ms = ?middleware_config.timeout_ms,
        body_limit_bytes = ?middleware_config.body_limit_bytes,
        "REST middleware plan resolved"
    );
    let app = route::router(ctx.clone()).layer(roze_http::middleware::AddExtensionLayer::new(
        roze_http::ws::WebSocketShutdown::new(service_shutdown),
    ));
    tracing::info!(
        event = "rest.router.constructed",
        protocol = "rest",
        "REST router constructed"
    );
    let app = middleware::app::apply(app, ctx);
    tracing::info!(
        event = "rest.middleware.application_applied",
        protocol = "rest",
        "application middleware hook applied"
    );
    let app = roze_middleware::apply_common_with_config(app, middleware_config);
    tracing::info!(
        event = "rest.middleware.common_applied",
        protocol = "rest",
        "Roze common middleware applied"
    );
    if rest.connect_info {
        group.add(RestService::new(
            service_name.clone(),
            RestServer::new(rest.addr, app).with_connect_info(),
        ));
    } else {
        group.add(RestService::new(
            service_name.clone(),
            RestServer::new(rest.addr, app),
        ));
    }
    group.add_fn("health-drain", move |shutdown| {
        let health = health.clone();
        async move {
            shutdown.wait().await;
            tracing::info!(
                event = roze_log::events::SERVICE_HEALTH_DRAINING,
                protocol = "rest",
                "shutdown requested; marking service draining"
            );
            health.mark_draining();
            Ok(())
        }
    });
    tracing::info!(event = roze_log::events::SERVICE_STARTING, service = %service_name, protocol = "rest", listen_addr = %rest.addr, "service starting");
    let result = group.start().await;
    if let Some(registration) = registration.as_mut() {
        registration.shutdown().await?;
        tracing::info!(event = roze_log::events::SERVICE_REGISTRY_UNREGISTERED, service = %service_name, protocol = "rest", "service unregistered");
    }
    match &result {
        Ok(()) => {
            tracing::info!(event = roze_log::events::SERVICE_STOPPED, service = %service_name, protocol = "rest", "service stopped")
        }
        Err(_) => {
            tracing::error!(event = roze_log::events::SERVICE_FAILED, service = %service_name, protocol = "rest", error_kind = "lifecycle", "service failed")
        }
    }
    result?;

    Ok(())
}

fn config_path() -> std::path::PathBuf {
    roze_config::service_config_path(env!("CARGO_MANIFEST_DIR"))
}
