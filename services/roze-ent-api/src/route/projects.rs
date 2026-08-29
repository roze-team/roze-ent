use roze_http::{
    routing::{delete, get, head, patch, post, put},
    Router,
};

use crate::handler;

pub fn routes() -> Router {
    Router::new()
        .route("/api/v1/projects", post(handler::projects::create_project))
        .route("/api/v1/projects/{id}", get(handler::projects::get_project))
        .route("/api/v1/projects", get(handler::projects::list_projects))
        .route(
            "/api/v1/projects/{id}",
            patch(handler::projects::update_project),
        )
        .route(
            "/api/v1/projects/{id}",
            delete(handler::projects::delete_project),
        )
}
