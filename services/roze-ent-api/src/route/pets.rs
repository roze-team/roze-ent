use roze_http::{
    routing::{delete, get, head, patch, post, put},
    Router,
};

use crate::handler;

pub fn routes() -> Router {
    Router::new()
        .route("/api/v1/pets", post(handler::pets::create_pet))
        .route("/api/v1/pets/{id}", get(handler::pets::get_pet))
        .route("/api/v1/pets/{id}", delete(handler::pets::delete_pet))
}
