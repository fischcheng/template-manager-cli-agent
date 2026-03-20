use std::path::Path;
use std::process::Command;

use crate::agent::AgentKind;
use crate::error::TmError;

pub trait SpecKitProvider {
    fn is_available(&self) -> Result<bool, TmError>;
    fn init(&self, cwd: &Path, agent: AgentKind) -> Result<SpecKitOutcome, TmError>;
}

#[derive(Debug, Clone)]
pub struct SpecKitOutcome {
    pub attempted: bool,
    pub succeeded: bool,
    pub message: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UvxSpecKitProvider;

impl UvxSpecKitProvider {
    pub fn command_args(agent: AgentKind) -> Vec<String> {
        vec![
            "--from".into(),
            "git+https://github.com/github/spec-kit.git".into(),
            "specify".into(),
            "init".into(),
            ".".into(),
            "--ai".into(),
            agent.to_string(),
            "--ignore-agent-tools".into(),
        ]
    }
}

impl SpecKitProvider for UvxSpecKitProvider {
    fn is_available(&self) -> Result<bool, TmError> {
        match Command::new("uvx").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    fn init(&self, cwd: &Path, agent: AgentKind) -> Result<SpecKitOutcome, TmError> {
        let mut command = Command::new("uvx");
        command.current_dir(cwd).args(Self::command_args(agent));

        let output = match command.output() {
            Ok(output) => output,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(SpecKitOutcome {
                    attempted: true,
                    succeeded: false,
                    message: "uvx is not installed; skipped Spec-Kit bootstrap".into(),
                });
            }
            Err(err) => {
                return Ok(SpecKitOutcome {
                    attempted: true,
                    succeeded: false,
                    message: format!("failed to start Spec-Kit: {err}"),
                });
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = match (stdout.is_empty(), stderr.is_empty()) {
            (false, true) => stdout,
            (true, false) => stderr,
            (false, false) => format!("{stdout}\n{stderr}"),
            (true, true) => String::new(),
        };

        if output.status.success() {
            Ok(SpecKitOutcome {
                attempted: true,
                succeeded: true,
                message: if detail.is_empty() {
                    "Spec-Kit bootstrap completed".into()
                } else {
                    detail
                },
            })
        } else {
            Ok(SpecKitOutcome {
                attempted: true,
                succeeded: false,
                message: if detail.is_empty() {
                    format!("Spec-Kit exited with status {}", output.status)
                } else {
                    detail
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::AgentKind;

    use super::UvxSpecKitProvider;

    #[test]
    fn builds_spec_kit_command() {
        let args = UvxSpecKitProvider::command_args(AgentKind::Gemini);
        assert_eq!(
            args,
            vec![
                "--from",
                "git+https://github.com/github/spec-kit.git",
                "specify",
                "init",
                ".",
                "--ai",
                "gemini",
                "--ignore-agent-tools",
            ]
        );
    }
}
