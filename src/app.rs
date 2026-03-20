use std::env;
use std::path::PathBuf;

use crate::agent::AgentKind;
use crate::cli::{Cli, Commands};
use crate::doctor;
use crate::error::TmError;
use crate::manifest::Manifest;
use crate::scaffold::{ScaffoldPlan, ScaffoldReport};
use crate::spec_kit::{SpecKitOutcome, SpecKitProvider, UvxSpecKitProvider};
use crate::update::{self, ManifestSource};

pub fn run(cli: Cli) -> Result<(), TmError> {
    let cwd = env::current_dir()?;
    let provider = UvxSpecKitProvider;

    match cli.command {
        Commands::Init {
            agent,
            lite,
            with_spec_kit,
        } => run_init(&cwd, agent, lite, with_spec_kit, &provider),
        Commands::Update {
            manifest_path,
            check,
        } => run_update(&cwd, manifest_path, check),
        Commands::Doctor => run_doctor(&cwd, &provider),
    }
}

fn run_init(
    cwd: &PathBuf,
    agent: AgentKind,
    lite: bool,
    with_spec_kit: bool,
    provider: &impl SpecKitProvider,
) -> Result<(), TmError> {
    let manifest = Manifest::embedded()?;
    let spec_kit = if with_spec_kit && !lite {
        Some(provider.init(cwd, agent)?)
    } else {
        None
    };

    let plan = ScaffoldPlan::build(cwd, manifest.agent(agent)?);
    let report = plan.apply()?;
    print_summary("init", agent, &report, spec_kit);
    Ok(())
}

fn run_update(cwd: &PathBuf, manifest_path: Option<PathBuf>, check: bool) -> Result<(), TmError> {
    let resolved = update::resolve_manifest(manifest_path.as_deref())?;
    for warning in &resolved.warnings {
        eprintln!("warning: {warning}");
    }

    let detected_agents: Vec<_> = AgentKind::ALL
        .into_iter()
        .filter(|agent| agent.detected_in(cwd))
        .collect();

    if detected_agents.is_empty() {
        println!(
            "update: no known agent scaffolds detected ({})",
            describe_manifest_source(resolved.source)
        );
        return Ok(());
    }

    let mut aggregate = ScaffoldReport::default();
    let mut total_actions = 0usize;

    for agent in detected_agents {
        let plan = ScaffoldPlan::build(cwd, resolved.manifest.agent(agent)?);
        total_actions += plan.action_count();
        if check {
            println!("update/{agent}: {} pending change(s)", plan.action_count());
            continue;
        }

        let report = plan.apply()?;
        aggregate.created_dirs += report.created_dirs;
        aggregate.created_files += report.created_files;
        aggregate.merged_files += report.merged_files;
        aggregate.skipped += report.skipped;
    }

    if check {
        if total_actions > 0 {
            return Err(TmError::CheckFailed(format!(
                "{total_actions} pending change(s) detected"
            )));
        }
        println!(
            "update: workspace is up to date ({})",
            describe_manifest_source(resolved.source)
        );
        return Ok(());
    }

    println!(
        "update: source={}, created_dirs={}, created_files={}, merged_files={}, skipped={}",
        describe_manifest_source(resolved.source),
        aggregate.created_dirs,
        aggregate.created_files,
        aggregate.merged_files,
        aggregate.skipped
    );
    Ok(())
}

fn run_doctor(cwd: &PathBuf, provider: &impl SpecKitProvider) -> Result<(), TmError> {
    let manifest = Manifest::embedded()?;
    let report = doctor::build_report(cwd, &manifest, provider)?;
    println!("{report}");
    Ok(())
}

fn print_summary(
    command: &str,
    agent: AgentKind,
    report: &ScaffoldReport,
    spec_kit: Option<SpecKitOutcome>,
) {
    println!(
        "{command}/{agent}: created_dirs={}, created_files={}, merged_files={}, skipped={}",
        report.created_dirs, report.created_files, report.merged_files, report.skipped
    );

    if let Some(outcome) = spec_kit {
        println!(
            "spec-kit: attempted={}, succeeded={}, message={}",
            outcome.attempted, outcome.succeeded, outcome.message
        );
    }
}

fn describe_manifest_source(source: ManifestSource) -> &'static str {
    match source {
        ManifestSource::Embedded => "embedded",
        ManifestSource::Cache => "cache",
        ManifestSource::ExplicitPath => "path",
        ManifestSource::Remote => "remote",
    }
}
