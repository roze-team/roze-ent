use std::{fs, path::Path};

use roze_ent::{
    generate_model_project_with_host, DependencySource, GenerateMode, GenerateOptions, HostAdapter,
    ModelFormat, ModelOrm, RozeDependency,
};

struct TestHost {
    dependency: RozeDependency,
}

impl HostAdapter for TestHost {
    fn roze_dependency(&self) -> Option<&RozeDependency> {
        Some(&self.dependency)
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
