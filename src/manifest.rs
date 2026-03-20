use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::agent::AgentKind;
use crate::error::TmError;

const EMBEDDED_MANIFEST: &str = include_str!("../assets/manifest.json");
const DEFAULT_REMOTE_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/yucheng/tm/main/assets/manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub agents: BTreeMap<AgentKind, AgentManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub root_file: Option<ManagedFile>,
    #[serde(default)]
    pub directories: Vec<ManagedDir>,
    #[serde(default)]
    pub files: Vec<ManagedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedDir {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedFile {
    pub path: String,
    pub content: String,
    pub policy: ManagedPolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedPolicy {
    CreateIfMissing,
    MergeJson,
    NeverTouchIfExists,
}

impl Manifest {
    pub fn embedded() -> Result<Self, TmError> {
        Self::from_json_str(EMBEDDED_MANIFEST)
    }

    pub fn from_path(path: &Path) -> Result<Self, TmError> {
        let content = fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    pub fn from_json_str(content: &str) -> Result<Self, TmError> {
        let manifest: Self = serde_json::from_str(content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), TmError> {
        if self.version.trim().is_empty() {
            return Err(TmError::InvalidManifest("version must not be empty".into()));
        }

        if self.agents.is_empty() {
            return Err(TmError::InvalidManifest(
                "manifest must define at least one agent".into(),
            ));
        }

        for (agent, config) in &self.agents {
            if let Some(root_file) = &config.root_file {
                validate_file(root_file, *agent)?;
            }
            for dir in &config.directories {
                if dir.path.trim().is_empty() {
                    return Err(TmError::InvalidManifest(format!(
                        "agent {agent} contains an empty directory path"
                    )));
                }
            }
            for file in &config.files {
                validate_file(file, *agent)?;
            }
        }

        Ok(())
    }

    pub fn agent(&self, agent: AgentKind) -> Result<&AgentManifest, TmError> {
        self.agents
            .get(&agent)
            .ok_or_else(|| TmError::MissingAgent(agent.to_string()))
    }
}

fn validate_file(file: &ManagedFile, agent: AgentKind) -> Result<(), TmError> {
    if file.path.trim().is_empty() {
        return Err(TmError::InvalidManifest(format!(
            "agent {agent} contains an empty file path"
        )));
    }

    if file.policy == ManagedPolicy::MergeJson && !file.path.ends_with(".json") {
        return Err(TmError::InvalidManifest(format!(
            "agent {agent} uses merge_json for non-json path {}",
            file.path
        )));
    }

    Ok(())
}

pub fn config_dir() -> Result<PathBuf, TmError> {
    let base_dirs = BaseDirs::new().ok_or(TmError::HomeDirectoryUnavailable)?;
    Ok(base_dirs.home_dir().join(".config").join("tm"))
}

pub fn cached_manifest_path() -> Result<PathBuf, TmError> {
    Ok(config_dir()?.join("manifest.json"))
}

pub fn cached_etag_path() -> Result<PathBuf, TmError> {
    Ok(config_dir()?.join("manifest.etag"))
}

pub fn remote_manifest_url() -> Option<String> {
    match std::env::var("TM_MANIFEST_URL") {
        Ok(value) if value.trim().is_empty() => None,
        Ok(value) => Some(value),
        Err(_) => Some(DEFAULT_REMOTE_MANIFEST_URL.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{ManagedPolicy, Manifest};

    #[test]
    fn parses_embedded_manifest() {
        let manifest = Manifest::embedded().unwrap();
        assert_eq!(manifest.version, "2026.03.20");
        assert!(
            manifest
                .agents
                .contains_key(&crate::agent::AgentKind::Claude)
        );
    }

    #[test]
    fn rejects_invalid_merge_target() {
        let manifest = r#"{
            "version":"1",
            "agents":{
                "codex":{
                    "directories":[],
                    "files":[{"path":"config.toml","content":"","policy":"merge_json"}]
                }
            }
        }"#;
        let error = Manifest::from_json_str(manifest).unwrap_err().to_string();
        assert!(error.contains("merge_json"));
    }

    #[test]
    fn deserializes_policy() {
        let policy: ManagedPolicy = serde_json::from_str(r#""create_if_missing""#).unwrap();
        assert_eq!(policy, ManagedPolicy::CreateIfMissing);
    }
}
