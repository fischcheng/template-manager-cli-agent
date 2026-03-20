use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("tm").unwrap()
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

    assert!(dir.path().join("CLAUDE.md").exists());
    assert!(dir.path().join(".claude/skills").exists());
    assert!(dir.path().join(".claude/workflows").exists());
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

    assert!(dir.path().join("AGENTS.md").exists());
    assert!(dir.path().join(".agents/skills").exists());
}

#[test]
fn update_uses_embedded_manifest_when_remote_is_disabled() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "custom\n").unwrap();

    bin()
        .current_dir(dir.path())
        .env("HOME", dir.path())
        .env("TM_MANIFEST_URL", "")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("source=embedded"));

    assert!(dir.path().join(".agents/skills").exists());
    assert!(dir.path().join(".agents/subagents").exists());
}

#[test]
fn update_check_reports_missing_files() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("GEMINI.md"), "custom\n").unwrap();

    bin()
        .current_dir(dir.path())
        .env("HOME", dir.path())
        .env("TM_MANIFEST_URL", "")
        .args(["update", "--check"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(
            "update/gemini: 2 pending change(s)",
        ));
}

#[test]
fn doctor_reports_workspace_status() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("CLAUDE.md"), "custom\n").unwrap();

    bin().current_dir(dir.path()).assert().failure();

    bin()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace/claude"))
        .stdout(predicate::str::contains("spec-kit:"));
}
