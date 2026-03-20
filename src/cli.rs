use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::agent::AgentKind;

#[derive(Debug, Parser)]
#[command(
    name = "tm",
    version,
    about = "Agent workspace scaffolding with optional Spec-Kit bootstrap"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        agent: AgentKind,
        #[arg(
            long,
            help = "Skip all external tool execution and use internal scaffolding only"
        )]
        lite: bool,
        #[arg(long, help = "Run Spec-Kit before applying tm normalization")]
        with_spec_kit: bool,
    },
    Update {
        #[arg(long, help = "Use a manifest file instead of remote/cache resolution")]
        manifest_path: Option<PathBuf>,
        #[arg(long, help = "Report pending changes without writing files")]
        check: bool,
    },
    Doctor,
}
