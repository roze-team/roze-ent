//! Stable library API for Roze model schemas and deterministic code generation.
//!
//! This crate deliberately does not depend on `rozectl`. CLI and project-specific
//! service wiring are supplied by a [`HostAdapter`].

mod host;
mod plan;

pub mod model;

pub use host::{
    DependencySource, GenerateMode, GenerateOptions, GeneratedDependency, HostAdapter,
    ModelBackend, ModelGenerationResult, ModelProjectRequirements, NoopHostAdapter, RozeDependency,
    RuntimeCapability, MODEL_PROJECT_REQUIREMENTS_API_VERSION,
};
pub use model::*;
