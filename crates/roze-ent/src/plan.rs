use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context};

use crate::GenerateMode;

static PLAN_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct GenerationPlan {
    target: PathBuf,
    workspace: PathBuf,
    staged: PathBuf,
}

impl GenerationPlan {
    #[allow(dead_code)]
    pub(crate) fn prepare(target: &Path, mode: GenerateMode) -> anyhow::Result<Self> {
        Self::prepare_inner(target, Some(mode))
    }

    pub(crate) fn prepare_component(target: &Path) -> anyhow::Result<Self> {
        Self::prepare_inner(target, None)
    }

    fn prepare_inner(target: &Path, project_mode: Option<GenerateMode>) -> anyhow::Result<Self> {
        let target = absolute_target(target)?;
        if target.exists()
            && project_mode == Some(GenerateMode::Create)
            && fs::read_dir(&target)
                .with_context(|| format!("failed to read {}", target.display()))?
                .next()
                .is_some()
        {
            bail!(
                "{} already exists and is not empty; pass --update to preserve business files or --force to overwrite all generated files",
                target.display()
            );
        }

        let parent = target
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let name = target
            .file_name()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| std::ffi::OsStr::new("project"));
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let sequence = PLAN_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let workspace = parent.join(format!(
            ".rozectl-plan-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        let staged = workspace.join(name);
        fs::create_dir_all(&workspace).with_context(|| {
            format!(
                "failed to create generation workspace {}",
                workspace.display()
            )
        })?;
        if target.exists() {
            copy_project(&target, &staged)?;
        } else {
            fs::create_dir_all(&staged)
                .with_context(|| format!("failed to create {}", staged.display()))?;
        }

        Ok(Self {
            target,
            workspace,
            staged,
        })
    }

    pub(crate) fn staged(&self) -> &Path {
        &self.staged
    }

    pub(crate) fn commit(self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir().context("failed to read current directory")?;
        let restore_relative = current_dir
            .strip_prefix(&self.target)
            .ok()
            .map(Path::to_path_buf);
        if restore_relative.is_some() {
            std::env::set_current_dir(
                self.target
                    .parent()
                    .context("generation target has no parent directory")?,
            )
            .with_context(|| {
                format!(
                    "failed to leave generation target {} before commit",
                    self.target.display()
                )
            })?;
        }

        let result = self.commit_project();
        if let Some(relative) = restore_relative {
            let restore = self.target.join(relative);
            std::env::set_current_dir(&restore).with_context(|| {
                format!(
                    "failed to restore current directory {} after generation",
                    restore.display()
                )
            })?;
        }
        result
    }

    fn commit_project(&self) -> anyhow::Result<()> {
        let backup = self.workspace.join("previous");
        if self.target.exists() {
            fs::rename(&self.target, &backup).with_context(|| {
                format!(
                    "failed to move {} to generation backup {}",
                    self.target.display(),
                    backup.display()
                )
            })?;
        }

        if let Err(error) = fs::rename(&self.staged, &self.target) {
            if backup.exists() {
                fs::rename(&backup, &self.target).with_context(|| {
                    format!(
                        "failed to restore {} after generation commit error: {error}",
                        self.target.display()
                    )
                })?;
            }
            return Err(error).with_context(|| {
                format!(
                    "failed to commit generated project {}",
                    self.target.display()
                )
            });
        }

        if backup.exists() {
            let _ = fs::remove_dir_all(&backup);
        }
        Ok(())
    }
}

impl Drop for GenerationPlan {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workspace);
    }
}

fn copy_project(source: &Path, destination: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if should_skip(&path) {
            continue;
        }
        let next = destination.join(name);
        if path.is_dir() {
            copy_project(&path, &next)?;
        } else if path.is_file() {
            fs::copy(&path, &next).with_context(|| {
                format!("failed to copy {} to {}", path.display(), next.display())
            })?;
        }
    }
    Ok(())
}

fn should_skip(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name == ".git"
        || name == "target"
        || name.starts_with(".rozectl-plan-")
        || name.starts_with(".rozectl-diff-")
}

fn absolute_target(target: &Path) -> anyhow::Result<PathBuf> {
    if target.exists() {
        return target
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", target.display()));
    }
    let absolute = if target.is_absolute() {
        target.to_path_buf()
    } else {
        std::env::current_dir()?.join(target)
    };
    let parent = absolute
        .parent()
        .context("generation target has no parent directory")?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let parent = parent
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", parent.display()))?;
    Ok(parent.join(
        absolute
            .file_name()
            .context("generation target must name a project directory")?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_target(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ))
    }

    #[test]
    fn dropped_plan_leaves_existing_project_unchanged() {
        let target = temp_target("rozectl-plan-rollback");
        fs::create_dir_all(&target).expect("create target");
        fs::write(target.join("owned.txt"), "original").expect("write original");

        let plan = GenerationPlan::prepare(&target, GenerateMode::Update).expect("prepare");
        fs::write(plan.staged().join("owned.txt"), "partial").expect("write staged");
        fs::write(plan.staged().join("new.txt"), "partial").expect("write staged new");
        drop(plan);

        assert_eq!(
            fs::read_to_string(target.join("owned.txt")).expect("read target"),
            "original"
        );
        assert!(!target.join("new.txt").exists());
        fs::remove_dir_all(target).expect("remove target");
    }

    #[test]
    fn committed_plan_replaces_project_and_preserves_application_files() {
        let target = temp_target("rozectl-plan-commit");
        fs::create_dir_all(&target).expect("create target");
        fs::write(target.join("application.txt"), "keep").expect("write application");
        fs::write(target.join("generated.txt"), "old").expect("write generated");
        fs::create_dir_all(target.join("target-assets")).expect("create application assets");
        fs::write(target.join("target-assets/logo.txt"), "keep").expect("write application asset");

        let plan = GenerationPlan::prepare(&target, GenerateMode::Update).expect("prepare");
        fs::write(plan.staged().join("generated.txt"), "new").expect("write staged");
        plan.commit().expect("commit");

        assert_eq!(
            fs::read_to_string(target.join("application.txt")).expect("read application"),
            "keep"
        );
        assert_eq!(
            fs::read_to_string(target.join("generated.txt")).expect("read generated"),
            "new"
        );
        assert_eq!(
            fs::read_to_string(target.join("target-assets/logo.txt"))
                .expect("read application asset"),
            "keep"
        );
        fs::remove_dir_all(target).expect("remove target");
    }
}
