//! Stable library API for Roze model schemas and deterministic code generation.
//!
//! This crate deliberately does not depend on `rozectl`. CLI and project-specific
//! service wiring are supplied by a [`HostAdapter`].

mod host;
mod plan;

pub mod model;

pub use host::{
    DependencySource, GenerateMode, GenerateOptions, HostAdapter, NoopHostAdapter, RozeDependency,
};
pub use model::*;
