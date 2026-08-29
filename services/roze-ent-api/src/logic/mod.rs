#![allow(dead_code, unused_imports)]

use roze_error::RozeError;

use crate::svc::ServiceContext;
use crate::types::*;

include!("prelude.rs");

pub fn current_subject(request_ctx: &roze_context::Context) -> Option<String> {
    request_ctx
        .subject()
        .or_else(|| request_ctx.metadata_value(roze_context::USER_ID_METADATA_KEY))
}

pub fn current_user_id(request_ctx: &roze_context::Context) -> Option<String> {
    current_subject(request_ctx)
}

pub fn current_admin_id(request_ctx: &roze_context::Context) -> Option<String> {
    current_subject(request_ctx)
}

pub fn current_tenant(request_ctx: &roze_context::Context) -> Option<String> {
    request_ctx.tenant()
}

pub fn current_roles(request_ctx: &roze_context::Context) -> Vec<String> {
    request_ctx.roles()
}

pub fn current_permissions(request_ctx: &roze_context::Context) -> Vec<String> {
    request_ctx.permissions()
}

pub fn current_scope(request_ctx: &roze_context::Context) -> Option<String> {
    request_ctx.metadata_value(roze_context::SCOPE_METADATA_KEY)
}

pub mod groups;
pub use groups::*;
pub mod pets;
pub use pets::*;
pub mod users;
pub use users::*;
