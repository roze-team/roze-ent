use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

use roze_ent::{
    generate_model_project_with_extensions_and_host_result, generate_model_project_with_host,
    generate_model_project_with_host_result, inspect_model_project_with_host_result, model_graph,
    model_project_requirements, DependencySource, GenerateMode, GenerateOptions,
    GeneratedDependency, HostAdapter, InspectDatabaseKind, ModelBackend, ModelFormat,
    ModelGenerationGraph, ModelGeneratorExtension, ModelOrm, ModelProjectRequirements,
    RozeDependency, RuntimeCapability, MODEL_PROJECT_REQUIREMENTS_API_VERSION,
};

struct TestHost {
    dependency: RozeDependency,
}

impl HostAdapter for TestHost {
    fn roze_dependency(&self) -> Option<&RozeDependency> {
        Some(&self.dependency)
    }
}

struct RequirementsHost {
    dependency: RozeDependency,
    seen: Mutex<Vec<ModelProjectRequirements>>,
    fail: bool,
}

impl RequirementsHost {
    fn new(fail: bool) -> Self {
        Self {
            dependency: RozeDependency::pinned("https://example.invalid/roze.git", "fixed-rev")
                .unwrap(),
            seen: Mutex::new(Vec::new()),
            fail,
        }
    }
}

impl HostAdapter for RequirementsHost {
    fn roze_dependency(&self) -> Option<&RozeDependency> {
        Some(&self.dependency)
    }

    fn sync_model_project(
        &self,
        _staged_project: &Path,
        requirements: &ModelProjectRequirements,
    ) -> anyhow::Result<()> {
        self.seen.lock().unwrap().push(requirements.clone());
        anyhow::ensure!(!self.fail, "intentional host wiring failure");
        Ok(())
    }
}

struct LegacyHost {
    calls: AtomicUsize,
}

impl HostAdapter for LegacyHost {
    fn sync_project(&self, _staged_project: &Path) -> anyhow::Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn schema(name: &str, table: &str) -> String {
    format!(
        "entity {name} {{\n  table \"{table}\"\n  cache true\n\n  field id: i64 {{\n    primary\n  }}\n}}\n"
    )
}

fn initialize_project(out: &Path) {
    fs::create_dir_all(out.join("src")).unwrap();
    fs::write(
        out.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    )
    .unwrap();
    fs::write(out.join("src/main.rs"), "fn main() {}\n").unwrap();
}

#[test]
fn host_pin_is_shared_updates_are_stable_and_extensions_are_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("fixture");
    initialize_project(&out);
    let host = TestHost {
        dependency: RozeDependency::pinned("https://example.invalid/roze.git", "fixed-rev")
            .unwrap(),
    };
    let create = GenerateOptions::new(GenerateMode::Create, DependencySource::Git);
    generate_model_project_with_host(
        &schema("User", "users"),
        &out,
        create,
        ModelFormat::Ent,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap();

    let manifest = fs::read_to_string(out.join("Cargo.toml")).unwrap();
    for dependency in ["roze-orm", "roze-cache"] {
        let line = manifest
            .lines()
            .find(|line| line.starts_with(dependency))
            .unwrap();
        assert!(line.contains("https://example.invalid/roze.git"));
        assert!(line.contains("fixed-rev"));
    }

    let extension = out.join("src/model/user_ext.rs");
    fs::write(&extension, "// application-owned\n").unwrap();
    let update = GenerateOptions::new(GenerateMode::Update, DependencySource::Git);
    generate_model_project_with_host(
        &schema("User", "users"),
        &out,
        update,
        ModelFormat::Ent,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap();
    let first_manifest = fs::read(out.join("Cargo.toml")).unwrap();
    generate_model_project_with_host(
        &schema("User", "users"),
        &out,
        update,
        ModelFormat::Ent,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap();
    assert_eq!(fs::read(out.join("Cargo.toml")).unwrap(), first_manifest);
    assert_eq!(
        fs::read_to_string(&extension).unwrap(),
        "// application-owned\n"
    );

    generate_model_project_with_host(
        &schema("Account", "accounts"),
        &out,
        update,
        ModelFormat::Ent,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap();
    assert!(!out.join("src/model/user.rs").exists());
    assert!(extension.exists());
    assert!(out.join("src/model/account.rs").exists());
}

#[test]
fn host_rejects_a_conflicting_existing_roze_revision() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("fixture");
    initialize_project(&out);
    fs::write(
        out.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nroze-orm = { git = \"https://example.invalid/roze.git\", rev = \"wrong\" }\n",
    )
    .unwrap();
    let host = TestHost {
        dependency: RozeDependency::pinned("https://example.invalid/roze.git", "fixed-rev")
            .unwrap(),
    };
    let error = generate_model_project_with_host(
        &schema("User", "users"),
        &out,
        GenerateOptions::new(GenerateMode::Update, DependencySource::Git),
        ModelFormat::Ent,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap_err();
    assert!(error.to_string().contains("host requires"));
}

#[test]
fn mongo_update_cleans_marked_files_and_preserves_extensions() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("fixture");
    initialize_project(&out);
    let host = TestHost {
        dependency: RozeDependency::pinned("https://example.invalid/roze.git", "fixed-rev")
            .unwrap(),
    };
    let user = "model User {\n  table: users\n  primary: id\n  field id ObjectId\n}\n";
    generate_model_project_with_host(
        user,
        &out,
        GenerateOptions::new(GenerateMode::Create, DependencySource::Git),
        ModelFormat::Mongo,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap();
    let extension = out.join("src/model/user_ext.rs");
    fs::write(&extension, "// mongo application-owned\n").unwrap();

    let account = "model Account {\n  table: accounts\n  primary: id\n  field id ObjectId\n}\n";
    generate_model_project_with_host(
        account,
        &out,
        GenerateOptions::new(GenerateMode::Update, DependencySource::Git),
        ModelFormat::Mongo,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap();
    assert!(!out.join("src/model/user.rs").exists());
    assert_eq!(
        fs::read_to_string(extension).unwrap(),
        "// mongo application-owned\n"
    );
    assert!(out.join("src/model/account.rs").exists());
    assert!(out.join("src/model/account_ext.rs").exists());
}

#[test]
fn mongo_project_requirements_match_golden_contract() {
    assert_eq!(MODEL_PROJECT_REQUIREMENTS_API_VERSION, 2);
    let graph = model_graph(
        "model User {\n  table: users\n  primary: id\n  cache: true\n  field id ObjectId\n  field metadata serde_json::Value\n}\n",
        ModelFormat::Mongo,
        ModelOrm::SeaOrm,
    )
    .unwrap();
    let requirements = model_project_requirements(&graph, ModelBackend::MongoDb);
    assert_eq!(
        requirements,
        ModelProjectRequirements {
            backend: ModelBackend::MongoDb,
            cargo_dependencies: vec![
                GeneratedDependency::versioned("anyhow", "1"),
                GeneratedDependency::without_features("roze-cache"),
                GeneratedDependency::without_features("roze-mongo"),
                GeneratedDependency::with_version_req("serde", "1", ["derive"]),
                GeneratedDependency::versioned("serde_json", "1"),
            ],
            runtime_capabilities: vec![
                RuntimeCapability::MongoConnection,
                RuntimeCapability::CacheConnection,
                RuntimeCapability::HealthRegistration,
                RuntimeCapability::ModelContextHook,
            ],
        }
    );
}

#[test]
fn dependency_constructors_preserve_legacy_callers_and_sort_features() {
    assert_eq!(
        GeneratedDependency::new("example", ["z", "a", "z"]),
        GeneratedDependency {
            name: "example".to_string(),
            features: vec!["a".to_string(), "z".to_string()],
            version_req: None,
        }
    );
    assert_eq!(
        GeneratedDependency::with_version_req("example", "1.2", ["z", "a", "z"]),
        GeneratedDependency {
            name: "example".to_string(),
            features: vec!["a".to_string(), "z".to_string()],
            version_req: Some("1.2".to_string()),
        }
    );
}

fn assert_dependency_contract(
    requirements: &ModelProjectRequirements,
    name: &str,
    version_req: Option<&str>,
    features: &[&str],
) {
    let dependency = requirements
        .dependency(name)
        .unwrap_or_else(|| panic!("missing generated dependency `{name}`"));
    assert_eq!(dependency.version_req.as_deref(), version_req, "{name}");
    assert_eq!(
        dependency
            .features
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        features,
        "{name}"
    );
}

fn manifest_from_requirements(requirements: &ModelProjectRequirements) -> String {
    let mut manifest = String::from("[dependencies]\n");
    for dependency in &requirements.cargo_dependencies {
        manifest.push_str(&dependency.name);
        manifest.push_str(" = { ");
        if let Some(version_req) = &dependency.version_req {
            manifest.push_str(&format!("version = {version_req:?}"));
        } else {
            manifest.push_str("workspace = true");
        }
        if !dependency.features.is_empty() {
            let features = dependency
                .features
                .iter()
                .map(|feature| format!("{feature:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            manifest.push_str(&format!(", features = [{features}]"));
        }
        manifest.push_str(" }\n");
    }
    manifest
}

#[test]
fn every_backend_reports_versions_for_all_crates_io_dependencies() {
    let source = r#"
        entity Fixture {
            table "fixtures"
            cache true
            field id: i64 {
                primary
            }
            field token: string {
                default uuid_new_string
                match "^[a-z0-9-]+$"
            }
            field amount: decimal {
            }
            field metadata: json {
            }
            field local_time: timestamp {
            }
            field created_at: timestamptz {
            }
        }
    "#;

    for (orm, backend) in [
        (ModelOrm::SeaOrm, ModelBackend::SeaOrm),
        (ModelOrm::Toasty, ModelBackend::Toasty),
        (ModelOrm::SeaOrm, ModelBackend::MongoDb),
    ] {
        let graph = model_graph(source, ModelFormat::Ent, orm).unwrap();
        let requirements = model_project_requirements(&graph, backend);
        let manifest = manifest_from_requirements(&requirements);
        assert_eq!(manifest, manifest_from_requirements(&requirements));
        manifest.parse::<toml_edit::DocumentMut>().unwrap();
        assert!(requirements.cargo_dependencies.iter().all(|dependency| {
            dependency.name.starts_with("roze-") == dependency.version_req.is_none()
        }));
        assert_dependency_contract(&requirements, "anyhow", Some("1"), &[]);
        assert_dependency_contract(&requirements, "serde", Some("1"), &["derive"]);
        assert_dependency_contract(&requirements, "rust_decimal", Some("1"), &["serde"]);
        assert_dependency_contract(&requirements, "regex", Some("1"), &[]);
        assert_dependency_contract(&requirements, "uuid", Some("1"), &["v7"]);

        match backend {
            ModelBackend::SeaOrm => {
                assert_dependency_contract(
                    &requirements,
                    "chrono",
                    Some("0.4"),
                    &["clock", "serde"],
                );
                assert_dependency_contract(&requirements, "serde_json", Some("1"), &[]);
                assert_dependency_contract(
                    &requirements,
                    "sea-orm",
                    Some("1"),
                    &[
                        "macros",
                        "runtime-tokio-rustls",
                        "sqlx-mysql",
                        "sqlx-postgres",
                        "sqlx-sqlite",
                        "with-chrono",
                        "with-json",
                        "with-rust_decimal",
                    ],
                );
            }
            ModelBackend::Toasty => {
                assert_dependency_contract(&requirements, "jiff", Some("0.2"), &["serde"]);
                assert_dependency_contract(
                    &requirements,
                    "toasty",
                    Some("0.7"),
                    &["jiff", "mysql", "postgresql", "rust_decimal", "serde"],
                );
            }
            ModelBackend::MongoDb => {
                assert_dependency_contract(
                    &requirements,
                    "chrono",
                    Some("0.4"),
                    &["clock", "serde"],
                );
                assert_dependency_contract(&requirements, "serde_json", Some("1"), &[]);
            }
        }
    }
}

#[test]
fn sql_backend_requirements_preserve_sea_orm_and_toasty_contracts() {
    let sea_graph =
        model_graph(&schema("User", "users"), ModelFormat::Ent, ModelOrm::SeaOrm).unwrap();
    let sea = model_project_requirements(&sea_graph, ModelBackend::SeaOrm);
    assert!(sea.dependency("sea-orm").is_some());
    assert!(sea.dependency("roze-orm").is_some());
    assert!(sea.requires(RuntimeCapability::SqlConnection));

    let toasty_graph =
        model_graph(&schema("User", "users"), ModelFormat::Ent, ModelOrm::Toasty).unwrap();
    let toasty = model_project_requirements(&toasty_graph, ModelBackend::Toasty);
    assert!(toasty.dependency("toasty").is_some());
    assert!(toasty.dependency("roze-config").is_some());
    assert!(toasty.dependency("roze-db").is_some());
    assert!(toasty.dependency("roze-orm").is_some());
    assert!(toasty.requires(RuntimeCapability::SqlConnection));
}

struct EnableCache;

impl ModelGeneratorExtension for EnableCache {
    fn name(&self) -> &'static str {
        "enable-cache"
    }

    fn transform(&self, graph: &mut ModelGenerationGraph) -> anyhow::Result<()> {
        graph.models[0].cache = true;
        Ok(())
    }
}

#[test]
fn create_update_and_extensions_report_the_same_requirements() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("fixture");
    initialize_project(&out);
    let host = RequirementsHost::new(false);
    let source = "model User {\n  table: users\n  primary: id\n  field id ObjectId\n}\n";
    let extension = EnableCache;
    let create = generate_model_project_with_extensions_and_host_result(
        source,
        &out,
        GenerateOptions::new(GenerateMode::Create, DependencySource::Git),
        ModelFormat::Mongo,
        ModelOrm::SeaOrm,
        &[&extension],
        &host,
    )
    .unwrap();
    let first = fs::read(out.join("src/model/mod.rs")).unwrap();
    let update = generate_model_project_with_extensions_and_host_result(
        source,
        &out,
        GenerateOptions::new(GenerateMode::Update, DependencySource::Git),
        ModelFormat::Mongo,
        ModelOrm::SeaOrm,
        &[&extension],
        &host,
    )
    .unwrap();
    assert_eq!(create, update);
    assert_eq!(fs::read(out.join("src/model/mod.rs")).unwrap(), first);
    assert!(create.requirements.dependency("roze-mongo").is_some());
    assert!(create.requirements.dependency("roze-cache").is_some());
    assert_eq!(
        host.seen.lock().unwrap().as_slice(),
        &[create.requirements.clone(), update.requirements]
    );
}

#[test]
fn legacy_sync_project_callback_remains_compatible() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("fixture");
    initialize_project(&out);
    let host = LegacyHost {
        calls: AtomicUsize::new(0),
    };
    generate_model_project_with_host_result(
        &schema("User", "users"),
        &out,
        GenerateOptions::new(GenerateMode::Create, DependencySource::Git),
        ModelFormat::Ent,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap();
    assert_eq!(host.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn requirements_callback_failure_rolls_back_atomically() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("fixture");
    initialize_project(&out);
    fs::write(out.join("owned.txt"), "original").unwrap();
    let host = RequirementsHost::new(true);
    let error = generate_model_project_with_host_result(
        &schema("User", "users"),
        &out,
        GenerateOptions::new(GenerateMode::Update, DependencySource::Git),
        ModelFormat::Ent,
        ModelOrm::SeaOrm,
        &host,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("intentional host wiring failure"));
    assert_eq!(
        fs::read_to_string(out.join("owned.txt")).unwrap(),
        "original"
    );
    assert!(!out.join("src/model").exists());
}

#[tokio::test]
async fn sqlite_inspect_reports_the_same_sea_orm_requirements() {
    let temp = tempfile::tempdir().unwrap();
    let database = temp.path().join("inspect.db");
    let url = format!(
        "sqlite://{}?mode=rwc",
        database.to_string_lossy().replace('\\', "/")
    );
    let pool = sqlx::SqlitePool::connect(&url).await.unwrap();
    sqlx::raw_sql("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;

    let out = temp.path().join("fixture");
    initialize_project(&out);
    let host = RequirementsHost::new(false);
    let inspected = inspect_model_project_with_host_result(
        "users",
        None,
        &url,
        InspectDatabaseKind::Sqlite,
        1,
        &out,
        GenerateOptions::new(GenerateMode::Create, DependencySource::Git),
        ModelOrm::SeaOrm,
        &host,
    )
    .await
    .unwrap();
    assert_eq!(inspected.requirements.backend, ModelBackend::SeaOrm);
    assert!(inspected.requirements.dependency("sea-orm").is_some());
    assert!(inspected
        .requirements
        .requires(RuntimeCapability::SqlConnection));
    assert_eq!(
        host.seen.lock().unwrap().as_slice(),
        &[inspected.requirements]
    );
}
