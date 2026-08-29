#![allow(unused_imports)]

pub mod groups;
pub mod pets;
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
