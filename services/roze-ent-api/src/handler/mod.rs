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
