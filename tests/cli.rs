use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("tm").unwrap()
}

fn embedded_manifest() -> Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/manifest.json");
    let content = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn agent_root_file(agent: &str) -> String {
    embedded_manifest()["agents"][agent]["root_file"]["path"]
        .as_str()
        .unwrap()
        .to_string()
}

fn agent_directories(agent: &str) -> Vec<String> {
    embedded_manifest()["agents"][agent]["directories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap().to_string())
        .collect()
}

fn agent_file_paths(agent: &str) -> Vec<String> {
    embedded_manifest()["agents"][agent]["files"]
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .map(|entry| entry["path"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn init_claude_lite_scaffolds_workspace() {
    let dir = tempdir().unwrap();

    bin()
        .current_dir(dir.path())
        .args(["init", "claude", "--lite"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init/claude"));

    assert!(dir.path().join(agent_root_file("claude")).exists());
    for relative in agent_directories("claude") {
        assert!(dir.path().join(relative).exists());
    }
}

#[test]
fn init_codex_with_spec_kit_falls_back_when_uvx_missing() {
    let dir = tempdir().unwrap();

    bin()
        .current_dir(dir.path())
        .env("PATH", "")
        .args(["init", "codex", "--with-spec-kit"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "spec-kit: attempted=true, succeeded=false",
        ));

    assert!(dir.path().join(agent_root_file("codex")).exists());
    for relative in agent_directories("codex") {
        assert!(dir.path().join(relative).exists());
    }
    for relative in agent_file_paths("codex") {
        assert!(dir.path().join(relative).exists());
    }
}

#[test]
fn init_with_spec_kit_rejects_non_empty_directory_before_invocation() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("README.md"), "existing\n").unwrap();

    bin()
        .current_dir(dir.path())
        .args(["init", "codex", "--with-spec-kit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot run Spec-Kit in non-empty directory",
        ));

    assert!(!dir.path().join(agent_root_file("codex")).exists());
    for relative in agent_directories("codex") {
        assert!(!dir.path().join(relative).exists());
    }
    for relative in agent_file_paths("codex") {
        assert!(!dir.path().join(relative).exists());
    }
}

#[test]
fn update_uses_embedded_manifest_when_remote_is_disabled() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(agent_root_file("codex")), "custom\n").unwrap();

    bin()
        .current_dir(dir.path())
        .env("HOME", dir.path())
        .env("TM_MANIFEST_URL", "")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("source=embedded"));

    for relative in agent_directories("codex") {
        assert!(dir.path().join(relative).exists());
    }
    for relative in agent_file_paths("codex") {
        assert!(dir.path().join(relative).exists());
    }
}

#[test]
fn update_check_reports_missing_files() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(agent_root_file("gemini")), "custom\n").unwrap();
    let pending_changes = agent_directories("gemini").len() + agent_file_paths("gemini").len();

    bin()
        .current_dir(dir.path())
        .env("HOME", dir.path())
        .env("TM_MANIFEST_URL", "")
        .args(["update", "--check"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(format!(
            "update/gemini: {pending_changes} pending change(s)",
        )));
}

#[test]
fn doctor_reports_workspace_status() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(agent_root_file("claude")), "custom\n").unwrap();

    bin().current_dir(dir.path()).assert().failure();

    bin()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace/claude"))
        .stdout(predicate::str::contains("spec-kit:"));
}
