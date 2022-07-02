use std::fmt::Display;

use clap::Parser;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PorceletCliStatus {
    Success,
    #[allow(dead_code)]
    InvalidArguments,
    UnknownError,
}

impl PorceletCliStatus {
    pub fn status(&self) -> i32 {
        match self {
            PorceletCliStatus::Success => 0,
            PorceletCliStatus::InvalidArguments => -1,
            PorceletCliStatus::UnknownError => -99,
        }
    }

    pub fn success(&self) -> bool {
        match self {
            PorceletCliStatus::Success => true,
            _ => false
        }
    }
}

impl Display for PorceletCliStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PorceletCliStatus::Success => f.write_str("success"),
            PorceletCliStatus::InvalidArguments => f.write_str("could not parse command line arguments"),
            PorceletCliStatus::UnknownError => f.write_str("unknown"),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct PorceletCliArgs {
    #[clap(subcommand)]
    subcommand: PorceletCliSubcommand,
}

#[derive(clap::Subcommand, Debug)]
#[clap(author, version, about)]
pub enum PorceletCliSubcommand {
    /// Control the procelet agent.
    Agent {
        #[clap(subcommand)]
        agent_subcommand: PorceletAgentSubcommand,
    },
    /// Show the status of the porcelet agent.
    Status,
}

#[derive(clap::Subcommand, Debug)]
#[clap(author, version, about)]
pub enum PorceletAgentSubcommand {
    /// Install the porcelet agent service on the machine.
    Install,
    /// Uninstall the porcelet agent service on the machine.
    Uninstall,
    /// Start the porcelet agent service.
    Start,
    /// Stop the porcelet agent service.
    Stop,
    /// Run the porcelet agent service. This should not be used
    /// directly.
    #[clap(hide = true)]
    Run,
}

/// Porcelet CLI entry point.
/// 
/// If args is None, args are parsed from the command line.
/// If the args are invalid or a help command is specified this will
/// terminate the program.
pub async fn cli_main(args: Option<PorceletCliArgs>) -> anyhow::Result<PorceletCliStatus> {
    let args = args.unwrap_or(PorceletCliArgs::parse());

    Ok(PorceletCliStatus::Success)
}
