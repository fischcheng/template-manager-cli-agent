use std::path::{Path, PathBuf};

use crate::error::TmError;
use crate::fs;
use crate::manifest::{AgentManifest, ManagedFile, ManagedPolicy};

#[derive(Debug, Clone)]
pub struct ScaffoldPlan {
    actions: Vec<ScaffoldAction>,
}

#[derive(Debug, Clone)]
pub enum ScaffoldAction {
    CreateDir { path: PathBuf },
    CreateFile { path: PathBuf, content: String },
    MergeJson { path: PathBuf, content: String },
}

#[derive(Debug, Default, Clone)]
pub struct ScaffoldReport {
    pub created_dirs: usize,
    pub created_files: usize,
    pub merged_files: usize,
    pub skipped: usize,
}

impl ScaffoldPlan {
    pub fn build(root: &Path, agent_manifest: &AgentManifest) -> Self {
        let mut actions = Vec::new();

        for directory in &agent_manifest.directories {
            let path = root.join(&directory.path);
            if !path.exists() {
                actions.push(ScaffoldAction::CreateDir { path });
            }
        }

        if let Some(root_file) = &agent_manifest.root_file {
            plan_file(root, root_file, &mut actions);
        }

        for file in &agent_manifest.files {
            plan_file(root, file, &mut actions);
        }

        Self { actions }
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    #[cfg(test)]
    pub fn actions(&self) -> &[ScaffoldAction] {
        &self.actions
    }

    pub fn apply(&self) -> Result<ScaffoldReport, TmError> {
        let mut report = ScaffoldReport::default();

        for action in &self.actions {
            match action {
                ScaffoldAction::CreateDir { path } => {
                    if fs::ensure_dir(path)? {
                        report.created_dirs += 1;
                    } else {
                        report.skipped += 1;
                    }
                }
                ScaffoldAction::CreateFile { path, content } => {
                    if fs::write_file_if_missing(path, content)? {
                        report.created_files += 1;
                    } else {
                        report.skipped += 1;
                    }
                }
                ScaffoldAction::MergeJson { path, content } => {
                    if fs::merge_json_file(path, content)? {
                        report.merged_files += 1;
                    } else {
                        report.skipped += 1;
                    }
                }
            }
        }

        Ok(report)
    }
}

fn plan_file(root: &Path, file: &ManagedFile, actions: &mut Vec<ScaffoldAction>) {
    let path = root.join(&file.path);

    match file.policy {
        ManagedPolicy::CreateIfMissing | ManagedPolicy::NeverTouchIfExists => {
            if !path.exists() {
                actions.push(ScaffoldAction::CreateFile {
                    path,
                    content: file.content.clone(),
                });
            }
        }
        ManagedPolicy::MergeJson => {
            actions.push(ScaffoldAction::MergeJson {
                path,
                content: file.content.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::manifest::{AgentManifest, ManagedDir, ManagedFile, ManagedPolicy};

    use super::ScaffoldPlan;

    fn fixture_manifest() -> AgentManifest {
        AgentManifest {
            root_file: Some(ManagedFile {
                path: "AGENTS.md".into(),
                content: "root\n".into(),
                policy: ManagedPolicy::NeverTouchIfExists,
            }),
            directories: vec![ManagedDir {
                path: ".agents/skills".into(),
            }],
            files: vec![ManagedFile {
                path: ".gemini/agent.json".into(),
                content: "{ \"mcpServers\": {} }".into(),
                policy: ManagedPolicy::MergeJson,
            }],
        }
    }

    #[test]
    fn plans_for_empty_workspace() {
        let dir = tempdir().unwrap();
        let plan = ScaffoldPlan::build(dir.path(), &fixture_manifest());
        assert_eq!(plan.action_count(), 3);
    }

    #[test]
    fn skips_existing_user_owned_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "custom\n").unwrap();

        let plan = ScaffoldPlan::build(dir.path(), &fixture_manifest());

        assert_eq!(plan.action_count(), 2);
    }

    #[test]
    fn still_plans_merge_for_json_files() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gemini")).unwrap();
        std::fs::write(
            dir.path().join(".gemini/agent.json"),
            "{ \"mcpServers\": {} }",
        )
        .unwrap();

        let plan = ScaffoldPlan::build(dir.path(), &fixture_manifest());

        assert!(
            plan.actions()
                .iter()
                .any(|action| matches!(action, super::ScaffoldAction::MergeJson { .. }))
        );
    }
}
