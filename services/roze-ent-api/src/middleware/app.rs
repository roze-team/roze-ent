use roze_http::Router;

use crate::svc::ServiceContext;

/// Stable application-owned hook for service-wide middleware.
///
/// This file is preserved by `rozectl api generate --update`. Add custom
/// Tower/Roze HTTP layers here; Roze common middleware wraps the returned
/// router so request context and CORS preflight run before application layers.
pub fn apply(router: Router, ctx: ServiceContext) -> Router {
    let _ = ctx;
    router
}
