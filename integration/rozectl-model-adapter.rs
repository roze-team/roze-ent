//! `rozectl model` compatibility adapter.
//!
//! Keep this module intentionally thin: parsing, inspection, rendering, update
//! semantics, cleanup, and extensions are owned by the `roze-ent` crate.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::generator::{
    find_workspace_root, inherited_roze_dependency, local_crates_prefix,
    validate_roze_dependency_sources, DependencySource, GenerateMode, GenerateOptions,
};

pub use roze_ent::{
    model_graph, normalize_model_source_to_ent, parse_models, parse_models_with_format,
    validate_model_graph, ExtensionFileOwnership, InspectDatabaseKind, ModelAnnotation,
    ModelBackend, ModelEdge, ModelExtensionFile, ModelField, ModelFieldValidation, ModelFormat,
    ModelGenerationGraph, ModelGeneratorExtension, ModelIndex, ModelOrm, ModelProjectRequirements,
    ModelSpec, ModelThroughEdge, RuntimeCapability, MODEL_GENERATOR_EXTENSION_API_VERSION,
};

struct RozectlHost {
    dependency: roze_ent::RozeDependency,
    logical_out: PathBuf,
    dependency_source: DependencySource,
}

impl RozectlHost {
    fn current(logical_out: &Path, dependency_source: DependencySource) -> anyhow::Result<Self> {
        Ok(Self {
            dependency: roze_ent::RozeDependency::pinned(super::ROZE_GIT_URL, super::ROZE_GIT_REV)?,
            logical_out: logical_out.to_path_buf(),
            dependency_source,
        })
    }
}

impl roze_ent::HostAdapter for RozectlHost {
    fn roze_dependency(&self) -> Option<&roze_ent::RozeDependency> {
        Some(&self.dependency)
    }

    fn sync_model_project(
        &self,
        staged_project: &Path,
        requirements: &ModelProjectRequirements,
    ) -> anyhow::Result<()> {
        if requirements.backend == ModelBackend::MongoDb {
            anyhow::ensure!(
                requirements.dependency("roze-mongo").is_some(),
                "Mongo model requirements must declare the direct `roze-mongo` dependency"
            );
            anyhow::ensure!(
                requirements.requires(RuntimeCapability::MongoConnection)
                    && requirements.requires(RuntimeCapability::HealthRegistration)
                    && requirements.requires(RuntimeCapability::ModelContextHook),
                "Mongo model requirements are missing runtime wiring capabilities"
            );
            ensure_mongo_project_wiring(staged_project, &self.logical_out, self.dependency_source)?;
        }
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
    let host = RozectlHost::current(out, generate_options.dependency_source)?;
    roze_ent::generate_model_project_with_host(
        source,
        out,
        options(generate_options),
        format,
        orm,
        &host,
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
    let host = RozectlHost::current(out, generate_options.dependency_source)?;
    roze_ent::generate_model_project_with_extensions_and_host(
        source,
        out,
        options(generate_options),
        format,
        orm,
        extensions,
        &host,
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
    let host = RozectlHost::current(out, generate_options.dependency_source)?;
    roze_ent::inspect_model_project_with_host(
        table,
        schema_name,
        db_url,
        db_kind,
        sample_size,
        out,
        options(generate_options),
        orm,
        &host,
    )
    .await
}

pub(super) fn is_mongo_model_project(out: &Path) -> bool {
    project_declares_dependency(out, "roze-mongo").unwrap_or(false)
}

pub(super) fn ensure_mongo_project_wiring(
    staged_out: &Path,
    logical_out: &Path,
    source: DependencySource,
) -> anyhow::Result<()> {
    update_mongo_model_context_hook(staged_out)?;
    update_mongo_service_context(staged_out)?;
    update_mongo_dependency(staged_out, logical_out, source)
}

fn update_mongo_model_context_hook(out: &Path) -> anyhow::Result<()> {
    let module_path = out.join("src/model/mod.rs");
    if !module_path.is_file() {
        return Ok(());
    }
    let mut module = fs::read_to_string(&module_path)
        .with_context(|| format!("failed to read {}", module_path.display()))?;
    if !module.contains("pub async fn configure_context(") {
        module.push_str(
            "\nuse crate::svc::ServiceContext;\n\npub async fn configure_context(\n    ctx: ServiceContext,\n) -> anyhow::Result<ServiceContext> {\n    Ok(ctx)\n}\n",
        );
    }
    fs::write(&module_path, module)
        .with_context(|| format!("failed to write {}", module_path.display()))
}

fn update_mongo_service_context(out: &Path) -> anyhow::Result<()> {
    let service_context_path = out.join("src/svc/mod.rs");
    if !service_context_path.is_file() {
        return Ok(());
    }
    let mut source = fs::read_to_string(&service_context_path)
        .with_context(|| format!("failed to read {}", service_context_path.display()))?;
    if source.contains("pub mongo: Option<roze_mongo::MongoDatabase>") {
        return Ok(());
    }

    source = replace_required(
        source,
        "    pub db_shards: Option<roze_db::ShardedDatabase>,\n    pub cache:",
        "    pub db_shards: Option<roze_db::ShardedDatabase>,\n    pub mongo: Option<roze_mongo::MongoDatabase>,\n    pub cache:",
        &service_context_path,
    )?;
    source = replace_required(
        source,
        "            .and_then(roze_db::DatabaseRuntime::sharded)\n            .cloned();\n        let cache =",
        "            .and_then(roze_db::DatabaseRuntime::sharded)\n            .cloned();\n        let mongo = roze_mongo::connect_optional(config.mongo.as_ref()).await?;\n        let cache =",
        &service_context_path,
    )?;
    source = replace_required(
        source,
        "        if let Some(cache) = cache.clone() {",
        "        if let Some(mongo) = mongo.clone() {\n            health.register_dependency(\"mongo\", move || {\n                let mongo = mongo.clone();\n                async move { mongo.health_check().await }\n            });\n        }\n        if let Some(cache) = cache.clone() {",
        &service_context_path,
    )?;
    source = replace_required(
        source,
        "            db_connections,\n            db_shards,\n            cache,",
        "            db_connections,\n            db_shards,\n            mongo,\n            cache,",
        &service_context_path,
    )?;
    fs::write(&service_context_path, source)
        .with_context(|| format!("failed to write {}", service_context_path.display()))
}

fn replace_required(source: String, from: &str, to: &str, path: &Path) -> anyhow::Result<String> {
    anyhow::ensure!(
        source.contains(from),
        "cannot add Mongo wiring because the generated service context anchor is missing in {}",
        path.display()
    );
    Ok(source.replacen(from, to, 1))
}

fn update_mongo_dependency(
    staged_out: &Path,
    logical_out: &Path,
    source: DependencySource,
) -> anyhow::Result<()> {
    let manifest_path = staged_out.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Ok(());
    }

    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut document = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    let dependencies = document
        .get_mut("dependencies")
        .and_then(toml_edit::Item::as_table_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("{} has no [dependencies] table", manifest_path.display())
        })?;
    validate_roze_dependency_sources(dependencies)?;
    if dependencies.contains_key("roze-mongo") {
        return Ok(());
    }

    let inherited = inherited_roze_dependency(dependencies, "roze-mongo")?;
    let inherited = match inherited {
        Some(item)
            if !dependency_uses_workspace(&item)
                || workspace_declares_dependency(logical_out, "roze-mongo")? =>
        {
            Some(item)
        }
        _ => None,
    };
    let dependency = if let Some(inherited) = inherited {
        inherited
    } else {
        match source {
            DependencySource::Git => format!(
                r#"{{ git = "{}", rev = "{}" }}"#,
                super::ROZE_GIT_URL,
                super::ROZE_GIT_REV
            )
            .parse::<toml_edit::Item>()?,
            DependencySource::Path => {
                let workspace_root = find_workspace_root(logical_out)?.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--roze-source path requires output inside a Cargo workspace containing Roze crates"
                    )
                })?;
                let prefix = local_crates_prefix(logical_out, &workspace_root)?;
                format!(r#"{{ path = "{prefix}/roze-mongo" }}"#).parse::<toml_edit::Item>()?
            }
        }
    };
    dependencies.insert("roze-mongo", dependency);
    fs::write(&manifest_path, document.to_string())
        .with_context(|| format!("failed to write {}", manifest_path.display()))
}

fn dependency_uses_workspace(item: &toml_edit::Item) -> bool {
    item.as_inline_table()
        .and_then(|dependency| dependency.get("workspace"))
        .and_then(toml_edit::Value::as_bool)
        == Some(true)
}

fn workspace_declares_dependency(logical_out: &Path, name: &str) -> anyhow::Result<bool> {
    let Some(workspace_root) = find_workspace_root(logical_out)? else {
        return Ok(false);
    };
    let manifest_path = workspace_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let document = manifest
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    Ok(document
        .get("workspace")
        .and_then(toml_edit::Item::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml_edit::Item::as_table)
        .is_some_and(|dependencies| dependencies.contains_key(name)))
}

fn project_declares_dependency(out: &Path, name: &str) -> anyhow::Result<bool> {
    let manifest_path = out.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Ok(false);
    }
    let manifest = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let document = manifest
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    Ok(document
        .get("dependencies")
        .and_then(toml_edit::Item::as_table)
        .is_some_and(|dependencies| dependencies.contains_key(name)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_model_output(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
    }

    #[test]
    fn mongo_generation_adds_pinned_roze_dependency_and_updates_stably() {
        let out = temp_model_output("rozectl-mongo-dependency");
        fs::create_dir_all(out.join("src")).expect("create source directory");
        fs::write(out.join("src/main.rs"), "fn main() {}\n").expect("write main");
        fs::write(
            out.join("Cargo.toml"),
            r#"[package]
name = "mongo-service"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )
        .expect("write manifest");
        let source = r#"
model User {
    table: users
    primary: id
    field id object_id
    field username String
}
"#;

        generate_model_project(
            source,
            &out,
            GenerateOptions::new(GenerateMode::Create, DependencySource::Git),
            ModelFormat::Mongo,
            ModelOrm::SeaOrm,
        )
        .expect("generate Mongo model");

        let manifest = fs::read_to_string(out.join("Cargo.toml")).expect("read manifest");
        let expected = format!(
            r#"roze-mongo = {{ git = "{}", rev = "{}" }}"#,
            super::super::ROZE_GIT_URL,
            super::super::ROZE_GIT_REV
        );
        assert!(manifest.contains(&expected));
        assert!(is_mongo_model_project(&out));
        let model_mod = fs::read_to_string(out.join("src/model/mod.rs")).expect("read model mod");
        assert!(model_mod.contains("pub async fn configure_context("));

        generate_model_project(
            source,
            &out,
            GenerateOptions::new(GenerateMode::Update, DependencySource::Git),
            ModelFormat::Mongo,
            ModelOrm::SeaOrm,
        )
        .expect("update Mongo model");
        assert_eq!(
            fs::read_to_string(out.join("Cargo.toml")).expect("read updated manifest"),
            manifest
        );

        fs::remove_dir_all(out).expect("remove temporary model output");
    }

    #[test]
    fn mongo_generation_inherits_local_roze_dependency_source() {
        let root = temp_model_output("rozectl-mongo-path-dependency");
        let out = root.join("apps/mongo-service");
        fs::create_dir_all(out.join("src")).expect("create source directory");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = []\nresolver = \"2\"\n\n[workspace.dependencies]\nroze-http = { path = \"crates/roze-http\" }\n",
        )
        .expect("write workspace manifest");
        fs::write(out.join("src/main.rs"), "fn main() {}\n").expect("write main");
        fs::create_dir_all(out.join("src/svc")).expect("create service context directory");
        fs::write(
            out.join("src/svc/mod.rs"),
            r#"pub struct ServiceContext {
    pub db_connections: Option<roze_db::DatabaseConnections>,
    pub db_shards: Option<roze_db::ShardedDatabase>,
    pub cache: Option<roze_cache::RedisCache>,
}

async fn build(config: Config, health: Health) -> anyhow::Result<ServiceContext> {
    let database_runtime = roze_db::connect_runtime_optional(config.database.as_ref()).await?;
    let db_connections = database_runtime.as_ref().and_then(roze_db::DatabaseRuntime::direct).cloned();
    let db_shards = database_runtime
            .as_ref()
            .and_then(roze_db::DatabaseRuntime::sharded)
            .cloned();
        let cache = None;
        if let Some(cache) = cache.clone() {
            let _ = cache;
        }
        Ok(ServiceContext {
            db_connections,
            db_shards,
            cache,
        })
}
"#,
        )
        .expect("write service context");
        fs::write(
            out.join("Cargo.toml"),
            r#"[package]
name = "mongo-service"
version = "0.1.0"
edition = "2021"

[dependencies]
roze-http = { workspace = true }
"#,
        )
        .expect("write manifest");

        generate_model_project(
            r#"
model User {
    table: users
    primary: id
    field id object_id
}
"#,
            &out,
            GenerateOptions::new(GenerateMode::Create, DependencySource::Path),
            ModelFormat::Mongo,
            ModelOrm::SeaOrm,
        )
        .expect("generate Mongo model with local dependencies");

        let manifest = fs::read_to_string(out.join("Cargo.toml")).expect("read manifest");
        assert!(manifest.contains(r#"roze-mongo = { path = "../../crates/roze-mongo" }"#));
        let service_context =
            fs::read_to_string(out.join("src/svc/mod.rs")).expect("read service context");
        assert!(service_context.contains("pub mongo: Option<roze_mongo::MongoDatabase>"));
        assert!(
            service_context.contains("roze_mongo::connect_optional(config.mongo.as_ref()).await?")
        );
        assert!(service_context.contains("health.register_dependency(\"mongo\""));
        assert!(service_context.contains("            mongo,"));

        fs::remove_dir_all(root).expect("remove temporary workspace");
    }

    #[test]
    #[ignore = "compile-smoke: generates a REST+Mongo project and runs cargo check"]
    fn generated_rest_mongo_project_compiles_and_cross_updates_stably() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fixture_name = format!(
            "rozectl-rest-mongo-compile-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        );
        let root = workspace.join("target").join(&fixture_name);
        let out = workspace.join("apps").join(&fixture_name);
        fs::create_dir_all(&root).expect("create temporary root");
        let api = root.join("user.api");
        fs::write(
            &api,
            r#"
service user-api {
    @handler getUser
    get /users/:id (GetUserReq) returns (UserResp)
}

type GetUserReq {
    id string `path:"id"`
}

type UserResp {
    id string `json:"id"`
}
"#,
        )
        .expect("write API contract");

        crate::generator::registry()
            .dispatch(crate::generator::GeneratorCommand::ApiGenerate {
                api: api.clone(),
                out: out.clone(),
                options: GenerateOptions::new(GenerateMode::Create, DependencySource::Path),
            })
            .expect("generate REST project");

        let model = r#"
model User {
    table: users
    primary: id
    field id ObjectId
    field email String
    unique_index: email
}
"#;
        generate_model_project(
            model,
            &out,
            GenerateOptions::new(GenerateMode::Update, DependencySource::Path),
            ModelFormat::Mongo,
            ModelOrm::SeaOrm,
        )
        .expect("generate Mongo model");
        let first_manifest = fs::read(out.join("Cargo.toml")).expect("read first manifest");
        generate_model_project(
            model,
            &out,
            GenerateOptions::new(GenerateMode::Update, DependencySource::Path),
            ModelFormat::Mongo,
            ModelOrm::SeaOrm,
        )
        .expect("repeat Mongo update");
        assert_eq!(
            fs::read(out.join("Cargo.toml")).expect("read repeated manifest"),
            first_manifest
        );

        crate::generator::registry()
            .dispatch(crate::generator::GeneratorCommand::ApiGenerate {
                api,
                out: out.clone(),
                options: GenerateOptions::new(GenerateMode::Update, DependencySource::Path),
            })
            .expect("update REST after Mongo");
        assert!(is_mongo_model_project(&out));
        let service_context =
            fs::read_to_string(out.join("src/svc/mod.rs")).expect("read service context");
        assert!(service_context.contains("pub mongo: Option<roze_mongo::MongoDatabase>"));
        assert!(service_context.contains("health.register_dependency(\"mongo\""));
        assert!(fs::read_to_string(out.join("src/main.rs"))
            .expect("read generated entrypoint")
            .contains("model::configure_context(svc::ServiceContext::new(config).await?).await"));

        let manifest_path = out.join("Cargo.toml");
        let mut manifest = fs::read_to_string(&manifest_path).expect("read generated manifest");
        for (workspace_dependency, standalone_dependency) in [
            ("anyhow.workspace = true", "anyhow = \"1\""),
            (
                "config.workspace = true",
                "config = { version = \"0.15.24\", default-features = false, features = [\"json\", \"yaml\", \"toml\"] }",
            ),
            (
                "serde.workspace = true",
                "serde = { version = \"1\", features = [\"derive\"] }",
            ),
            ("serde_json.workspace = true", "serde_json = \"1\""),
            (
                "validator.workspace = true",
                "validator = { version = \"0.20\", features = [\"derive\"] }",
            ),
            (
                "tokio.workspace = true",
                "tokio = { version = \"1\", features = [\"macros\", \"rt-multi-thread\", \"signal\", \"sync\", \"time\"] }",
            ),
            ("tracing.workspace = true", "tracing = \"0.1\""),
            (
                "veil.workspace = true",
                "veil = { version = \"0.3.0\", default-features = false }",
            ),
        ] {
            manifest = manifest.replace(workspace_dependency, standalone_dependency);
        }
        manifest = manifest
            .replace("license.workspace = true", "license = \"Apache-2.0\"")
            .replace("version.workspace = true", "version = \"0.0.0\"");
        manifest.push_str("\n[workspace]\n");
        fs::write(&manifest_path, manifest).expect("write standalone smoke manifest");

        let output = Command::new("cargo")
            .args(["check", "--manifest-path"])
            .arg(&manifest_path)
            .output()
            .expect("run cargo check");
        assert!(
            output.status.success(),
            "generated Mongo project failed cargo check\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        fs::remove_dir_all(root).expect("remove temporary project");
        fs::remove_dir_all(out).expect("remove generated workspace member");
    }
}
