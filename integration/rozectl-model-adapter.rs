//! `rozectl model` compatibility adapter.
//!
//! Keep this module intentionally thin: parsing, inspection, rendering, update
//! semantics, cleanup, and extensions are owned by the `roze-ent` crate.

use std::path::{Path, PathBuf};

use crate::generator::{DependencySource, GenerateMode, GenerateOptions};

pub use roze_ent::{
    model_graph, normalize_model_source_to_ent, parse_models, parse_models_with_format,
    validate_model_graph, ExtensionFileOwnership, InspectDatabaseKind, ModelAnnotation, ModelEdge,
    ModelExtensionFile, ModelField, ModelFieldValidation, ModelFormat, ModelGenerationGraph,
    ModelGeneratorExtension, ModelIndex, ModelOrm, ModelSpec, ModelThroughEdge,
    MODEL_GENERATOR_EXTENSION_API_VERSION,
};

struct RozectlHost {
    dependency: roze_ent::RozeDependency,
}

impl RozectlHost {
    fn current() -> anyhow::Result<Self> {
        Ok(Self {
            dependency: roze_ent::RozeDependency::pinned(super::ROZE_GIT_URL, super::ROZE_GIT_REV)?,
        })
    }
}

impl roze_ent::HostAdapter for RozectlHost {
    fn roze_dependency(&self) -> Option<&roze_ent::RozeDependency> {
        Some(&self.dependency)
    }

    fn sync_project(&self, staged_project: &Path) -> anyhow::Result<()> {
        super::sync_managed_service_if_present(staged_project)
    }

    fn format_generated_rust(&self, project: &Path, rust_files: &[PathBuf]) -> anyhow::Result<()> {
        super::format_generated_rust_files(project, rust_files)
    }
}

fn options(options: GenerateOptions) -> roze_ent::GenerateOptions {
    roze_ent::GenerateOptions::new(
        match options.mode {
            GenerateMode::Create => roze_ent::GenerateMode::Create,
            GenerateMode::Update => roze_ent::GenerateMode::Update,
            GenerateMode::Force => roze_ent::GenerateMode::Force,
        },
        match options.dependency_source {
            DependencySource::Git => roze_ent::DependencySource::Git,
            DependencySource::Path => roze_ent::DependencySource::Path,
        },
    )
}

fn mode(mode: GenerateMode) -> roze_ent::GenerateMode {
    match mode {
        GenerateMode::Create => roze_ent::GenerateMode::Create,
        GenerateMode::Update => roze_ent::GenerateMode::Update,
        GenerateMode::Force => roze_ent::GenerateMode::Force,
    }
}

pub fn resolve_model_orm(
    out: &Path,
    generate_mode: GenerateMode,
    requested: Option<ModelOrm>,
    switch_orm: bool,
) -> anyhow::Result<ModelOrm> {
    roze_ent::resolve_model_orm(out, mode(generate_mode), requested, switch_orm)
}

pub fn generate_model_project(
    source: &str,
    out: &Path,
    generate_options: GenerateOptions,
    format: ModelFormat,
    orm: ModelOrm,
) -> anyhow::Result<()> {
    roze_ent::generate_model_project_with_host(
        source,
        out,
        options(generate_options),
        format,
        orm,
        &RozectlHost::current()?,
    )
}

pub fn generate_model_project_with_extensions(
    source: &str,
    out: &Path,
    generate_options: GenerateOptions,
    format: ModelFormat,
    orm: ModelOrm,
    extensions: &[&dyn ModelGeneratorExtension],
) -> anyhow::Result<()> {
    roze_ent::generate_model_project_with_extensions_and_host(
        source,
        out,
        options(generate_options),
        format,
        orm,
        extensions,
        &RozectlHost::current()?,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn inspect_model_project(
    table: &str,
    schema_name: Option<&str>,
    db_url: &str,
    db_kind: InspectDatabaseKind,
    sample_size: u64,
    out: &Path,
    generate_options: GenerateOptions,
    orm: ModelOrm,
) -> anyhow::Result<()> {
    roze_ent::inspect_model_project_with_host(
        table,
        schema_name,
        db_url,
        db_kind,
        sample_size,
        out,
        options(generate_options),
        orm,
        &RozectlHost::current()?,
    )
    .await
}
