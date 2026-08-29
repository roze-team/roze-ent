use roze_http::{
    routing::{delete, get, head, patch, post, put},
    Router,
};

use crate::handler;

pub fn routes() -> Router {
    Router::new()
        .route("/api/v1/users", post(handler::users::create_user))
        .route("/api/v1/users/{id}", get(handler::users::get_user))
        .route("/api/v1/users", get(handler::users::list_users))
        .route("/api/v1/users/{id}", delete(handler::users::delete_user))
        .route(
            "/api/v1/users/{id}/pets",
            get(handler::users::list_user_pets),
        )
        .route(
            "/api/v1/users/{id}/groups",
            get(handler::users::list_user_groups),
        )
}
