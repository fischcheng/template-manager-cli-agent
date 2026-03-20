use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::TmError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    Claude,
    Codex,
    Gemini,
}

impl AgentKind {
    pub const ALL: [AgentKind; 3] = [AgentKind::Claude, AgentKind::Codex, AgentKind::Gemini];

    pub fn marker_paths(self) -> &'static [&'static str] {
        match self {
            Self::Claude => &["CLAUDE.md", ".claude"],
            Self::Codex => &["AGENTS.md", ".agents"],
            Self::Gemini => &["GEMINI.md", ".gemini"],
        }
    }

    pub fn detected_in(self, root: &Path) -> bool {
        self.marker_paths()
            .iter()
            .any(|relative| root.join(relative).exists())
    }
}

impl fmt::Display for AgentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        };
        f.write_str(value)
    }
}

impl FromStr for AgentKind {
    type Err = TmError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_ascii_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "gemini" => Ok(Self::Gemini),
            _ => Err(TmError::InvalidAgent(input.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AgentKind;

    #[test]
    fn parses_agents() {
        assert_eq!("claude".parse::<AgentKind>().unwrap(), AgentKind::Claude);
        assert_eq!("codex".parse::<AgentKind>().unwrap(), AgentKind::Codex);
        assert_eq!("gemini".parse::<AgentKind>().unwrap(), AgentKind::Gemini);
    }

    #[test]
    fn formats_agents() {
        assert_eq!(AgentKind::Claude.to_string(), "claude");
        assert_eq!(AgentKind::Codex.to_string(), "codex");
        assert_eq!(AgentKind::Gemini.to_string(), "gemini");
    }
}
