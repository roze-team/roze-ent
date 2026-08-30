#![allow(unused_imports)]

pub mod groups;
pub mod pets;
pub mod projects;
pub mod users;

use roze_context::Context;
use roze_error::RozeError;
use roze_http::{
    extract::{Extension, Form, Path, Query, State},
    http::HeaderMap,
    Json,
};
use roze_result::ApiResponse;
use roze_validation::Validate;
use serde::Deserialize;

use crate::svc::ServiceContext;
use crate::types::*;

fn header_value<T>(headers: &HeaderMap, name: &str) -> Result<T, RozeError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let raw = headers
        .get(name)
        .ok_or_else(|| RozeError::BadRequest(format!("missing header `{name}`")))?;
    let raw = raw
        .to_str()
        .map_err(|err| RozeError::BadRequest(format!("invalid header `{name}`: {err}")))?;
    raw.parse::<T>()
        .map_err(|err| RozeError::BadRequest(format!("invalid header `{name}`: {err}")))
}

#[allow(dead_code)]
fn optional_header_value<T>(headers: &HeaderMap, name: &str) -> Result<Option<T>, RozeError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let Some(raw) = headers.get(name) else {
        return Ok(None);
    };
    let raw = raw
        .to_str()
        .map_err(|err| RozeError::BadRequest(format!("invalid header `{name}`: {err}")))?;
    raw.parse::<T>()
        .map(Some)
        .map_err(|err| RozeError::BadRequest(format!("invalid header `{name}`: {err}")))
}

fn authorize(
    headers: &HeaderMap,
    ctx: &ServiceContext,
    request_ctx: &roze_context::Context,
) -> Result<roze_context::Context, RozeError> {
    let jwt = ctx.jwt_config().ok_or(RozeError::Unauthorized)?;
    let header_value = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(RozeError::Unauthorized)?;
    let token = roze_jwt::extract_bearer_token(header_value).ok_or(RozeError::Unauthorized)?;
    let claims = roze_jwt::verify_token(token, &jwt).map_err(|_| RozeError::Unauthorized)?;
    let auth = roze_context::AuthContext {
        subject: claims.sub,
        roles: claims.roles,
        tenant: claims.tenant,
    };
    Ok(request_ctx
        .with_auth(auth)
        .with_permissions(claims.permissions)
        .with_metadata(roze_context::SCOPE_METADATA_KEY, claims.scopes.join(",")))
}
