use std::path::Path;

use crate::agent::AgentKind;
use crate::error::TmError;
use crate::manifest::{Manifest, cached_manifest_path};
use crate::scaffold::ScaffoldPlan;
use crate::spec_kit::SpecKitProvider;

pub fn build_report(
    cwd: &Path,
    manifest: &Manifest,
    provider: &impl SpecKitProvider,
) -> Result<String, TmError> {
    let mut lines = Vec::new();

    let uvx_available = provider.is_available()?;
    lines.push(format!(
        "spec-kit: {}",
        if uvx_available {
            "uvx available"
        } else {
            "uvx unavailable"
        }
    ));

    let embedded_version = Manifest::embedded()?.version;
    let cache_version = match cached_manifest_path() {
        Ok(path) if path.exists() => Some(Manifest::from_path(&path)?.version),
        _ => None,
    };
    lines.push(match cache_version {
        Some(version) if version != embedded_version => {
            format!("manifest: embedded={embedded_version}, cached={version}")
        }
        Some(version) => {
            format!("manifest: embedded={embedded_version}, cached={version} (in sync)")
        }
        None => format!("manifest: embedded={embedded_version}, cached=missing"),
    });

    for agent in AgentKind::ALL {
        let agent_manifest = manifest.agent(agent)?;
        let plan = ScaffoldPlan::build(cwd, agent_manifest);
        if plan.is_empty() {
            lines.push(format!("workspace/{agent}: ready"));
        } else {
            lines.push(format!(
                "workspace/{agent}: {} managed change(s) missing",
                plan.action_count()
            ));
        }
    }

    Ok(lines.join("\n"))
}
