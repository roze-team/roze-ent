use roze_http::{
    routing::{delete, get, head, patch, post, put},
    Router,
};

use crate::handler;

pub fn routes() -> Router {
    Router::new()
        .route("/api/v1/groups", post(handler::groups::create_group))
        .route("/api/v1/groups/{id}", get(handler::groups::get_group))
        .route("/api/v1/groups", get(handler::groups::list_groups))
        .route(
            "/api/v1/groups/{group_id}/members/{user_id}",
            post(handler::groups::add_group_member),
        )
        .route(
            "/api/v1/groups/{group_id}/members/{user_id}",
            patch(handler::groups::update_group_member),
        )
        .route(
            "/api/v1/groups/{group_id}/members/{user_id}",
            delete(handler::groups::remove_group_member),
        )
        .route(
            "/api/v1/groups/{id}/users",
            get(handler::groups::list_group_users),
        )
}
