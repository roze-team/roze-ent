use std::{path::Path, process::Command};

use anyhow::{bail, Context};

pub(crate) const ROZE_GIT_URL: &str = "https://github.com/roze-team/roze.git";
// Compatibility fallback for the legacy convenience entrypoints. Embedders
// should use a HostAdapter and provide the revision of their Roze checkout.
pub(crate) const ROZE_GIT_REV: &str = "e4bf750dfa630ca4224318d1e7c72a818598a2d2";

/// How generated projects obtain Roze runtime crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencySource {
    Git,
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerateMode {
    Create,
    Update,
    Force,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerateOptions {
    pub mode: GenerateMode,
    pub dependency_source: DependencySource,
}

impl GenerateOptions {
    pub const fn new(mode: GenerateMode, dependency_source: DependencySource) -> Self {
        Self {
            mode,
            dependency_source,
        }
    }
}

/// Exact Git source inherited by every generated `roze-*` dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RozeDependency {
    pub git: String,
    pub rev: String,
}

impl RozeDependency {
    pub fn pinned(git: impl Into<String>, rev: impl Into<String>) -> anyhow::Result<Self> {
        let dependency = Self {
            git: git.into(),
            rev: rev.into(),
        };
        anyhow::ensure!(
            !dependency.git.trim().is_empty(),
            "Roze Git URL cannot be empty"
        );
        anyhow::ensure!(
            !dependency.rev.trim().is_empty(),
            "Roze Git revision cannot be empty"
        );
        Ok(dependency)
    }
}

/// Host-owned integration hooks. `rozectl` implements service-manifest and
/// `ServiceContext` synchronization here; standalone users may keep the no-op.
pub trait HostAdapter: Send + Sync {
    fn roze_dependency(&self) -> Option<&RozeDependency> {
        None
    }

    fn sync_project(&self, _staged_project: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    fn format_generated_rust(
        &self,
        project: &Path,
        rust_files: &[std::path::PathBuf],
    ) -> anyhow::Result<()> {
        format_generated_rust_files(project, rust_files)
    }
}

#[derive(Debug, Default)]
pub struct NoopHostAdapter;

impl HostAdapter for NoopHostAdapter {}

pub(crate) fn format_generated_rust_files(
    out: &Path,
    rust_files: &[std::path::PathBuf],
) -> anyhow::Result<()> {
    let mut rust_files = rust_files.to_vec();
    rust_files.sort();
    rust_files.dedup();
    if rust_files.is_empty() {
        return Ok(());
    }
    let status = Command::new("rustfmt")
        .args(["--edition", "2021"])
        .args(["--config", "skip_children=true"])
        .args(&rust_files)
        .status()
        .context("failed to run rustfmt for generated Rust files")?;
    if !status.success() {
        bail!("rustfmt failed for generated project at {}", out.display());
    }
    Ok(())
}

pub(crate) fn find_workspace_root(out: &Path) -> anyhow::Result<Option<std::path::PathBuf>> {
    let absolute_out = if out.is_absolute() {
        out.to_path_buf()
    } else {
        std::env::current_dir()?.join(out)
    };
    for directory in absolute_out.ancestors() {
        let manifest = directory.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&manifest)
            .with_context(|| format!("failed to read {}", manifest.display()))?;
        if content.lines().any(|line| line.trim() == "[workspace]") {
            return Ok(Some(directory.to_path_buf()));
        }
    }
    Ok(None)
}

pub(crate) fn local_crates_prefix(out: &Path, workspace_root: &Path) -> anyhow::Result<String> {
    let absolute_out = if out.is_absolute() {
        out.to_path_buf()
    } else {
        std::env::current_dir()?.join(out)
    };
    let relative = absolute_out.strip_prefix(workspace_root).with_context(|| {
        format!(
            "{} is not inside workspace {}",
            out.display(),
            workspace_root.display()
        )
    })?;
    let depth = relative.components().count();
    if depth == 0 {
        bail!("project output cannot be the workspace root");
    }
    Ok(format!("{}crates", "../".repeat(depth)))
}

pub(crate) fn inherited_roze_dependency(
    dependencies: &toml_edit::Table,
    target: &str,
) -> anyhow::Result<Option<toml_edit::Item>> {
    let mut candidates = Vec::new();
    let mut has_workspace_source = false;
    for (name, item) in dependencies {
        if !name.starts_with("roze-") || name == target {
            continue;
        }
        let Some(table) = item.as_inline_table() else {
            continue;
        };
        if table.get("workspace").and_then(toml_edit::Value::as_bool) == Some(true) {
            has_workspace_source = true;
            continue;
        }
        if let Some(git) = table.get("git").and_then(toml_edit::Value::as_str) {
            let mut inherited = toml_edit::InlineTable::new();
            inherited.insert("git", git.into());
            for key in ["rev", "tag", "branch"] {
                if let Some(value) = table.get(key).and_then(toml_edit::Value::as_str) {
                    inherited.insert(key, value.into());
                }
            }
            candidates.push(toml_edit::Item::Value(toml_edit::Value::InlineTable(
                inherited,
            )));
        } else if let Some(path) = table.get("path").and_then(toml_edit::Value::as_str) {
            let sibling = Path::new(path)
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(target)
                .to_string_lossy()
                .replace('\\', "/");
            candidates.push(format!(r#"{{ path = "{sibling}" }}"#).parse::<toml_edit::Item>()?);
        }
    }
    candidates.sort_by_key(ToString::to_string);
    candidates.dedup_by(|left, right| left.to_string() == right.to_string());
    if candidates.len() > 1 {
        let sources = candidates
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        bail!("conflicting Roze dependency sources in Cargo.toml: {sources}");
    }
    Ok(candidates.pop().or_else(|| {
        has_workspace_source
            .then(|| r#"{ workspace = true }"#.parse::<toml_edit::Item>().expect("valid item"))
    }))
}

pub(crate) fn validate_roze_dependency_sources(
    dependencies: &mut toml_edit::Table,
) -> anyhow::Result<()> {
    let mut pins = dependencies
        .iter()
        .filter(|(name, _)| name.starts_with("roze-"))
        .filter_map(|(_, item)| {
            let table = item.as_inline_table()?;
            let git = table.get("git")?.as_str()?.to_string();
            let pin = ["rev", "tag", "branch"].into_iter().find_map(|key| {
                table
                    .get(key)
                    .and_then(toml_edit::Value::as_str)
                    .map(|value| (key, value.to_string()))
            });
            Some((git, pin))
        })
        .collect::<Vec<_>>();
    pins.sort();
    pins.dedup();
    anyhow::ensure!(
        pins.len() <= 1,
        "conflicting Roze Git dependency pins in Cargo.toml: {pins:?}"
    );
    if let Some((git, Some((pin_key, pin_value)))) = pins.first() {
        for (name, item) in dependencies.iter_mut() {
            if !name.starts_with("roze-") {
                continue;
            }
            let Some(table) = item.as_inline_table_mut() else {
                continue;
            };
            if table.get("git").and_then(toml_edit::Value::as_str) != Some(git.as_str()) {
                continue;
            }
            for key in ["rev", "tag", "branch"] {
                table.remove(key);
            }
            table.insert(*pin_key, pin_value.clone().into());
        }
    }
    Ok(())
}

pub(crate) fn to_pascal_case(input: &str) -> String {
    input
        .split(['_', '-', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

pub(crate) fn to_snake_case(input: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in input.chars().enumerate() {
        if ch.is_uppercase() && idx > 0 {
            out.push('_');
        } else if ch == '-' || ch == ' ' {
            out.push('_');
            continue;
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

pub(crate) fn rust_identifier(input: &str) -> String {
    let ident = to_snake_case(input);
    if matches!(
        ident.as_str(),
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    ) {
        format!("r#{ident}")
    } else {
        ident
    }
}
