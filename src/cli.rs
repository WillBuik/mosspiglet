use std::ffi::OsString;

use clap::Parser;
use windows_service::service_dispatcher;

use crate::{service::{SystemService, ServiceStatus, ServiceDescription}, agent::Agent, ffi_service_main};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct CliArgs {
    #[clap(subcommand)]
    subcommand: CliSubcommand,
}

#[derive(clap::Subcommand, Debug)]
#[clap(author, version, about)]
pub enum CliSubcommand {
    /// Control the procelet agent.
    Agent {
        #[clap(subcommand)]
        agent_subcommand: AgentSubcommand,
    },
    /// Show the status of the porcelet agent.
    Status,
}

#[derive(clap::Subcommand, Debug)]
#[clap(author, version, about)]
pub enum AgentSubcommand {
    /// Install the porcelet agent service on the machine.
    Install,
    /// Uninstall the porcelet agent service on the machine.
    Uninstall,
    /// Start the porcelet agent service.
    Start,
    /// Stop the porcelet agent service.
    Stop,
    /// Run the porcelet agent service as a process. This should
    /// not be used directly except for testing.
    #[clap(hide = true)]
    Run,
    /// Run the porcelet agent service as Windows service. Sets up the
    /// service dispatcher and message pump. This cannot be called from
    /// the command line because it needs to run in a Windows service
    /// context. Use 'run' for testing.
    #[clap(hide = true)]
    RunWindowsService,
}

async fn agent_command(agent_subcommand: AgentSubcommand) -> anyhow::Result<()> {
    let agent_service_manager = SystemService::new(Agent::SERVICE_NAME.into());

    match agent_subcommand {
        AgentSubcommand::Install => {
            println!("Installing Porcelet agent service...");

            let service_desc = ServiceDescription {
                friendly_name: Agent::SERVICE_DISPLAY_NAME.into(),
                binary_path: std::env::current_exe()?.into(),
                args: vec![OsString::from("agent"), OsString::from("run-windows-service")],
            };

            agent_service_manager.install(service_desc)?;
        },

        AgentSubcommand::Uninstall => {
            println!("Removing Porcelet agent service...");
            agent_service_manager.uninstall()?;
        },

        AgentSubcommand::Start => {
            println!("Starting Porcelet agent service...");
            agent_service_manager.start()?;
        },

        AgentSubcommand::Stop => {
            println!("Stopping Porcelet agent service...");
            agent_service_manager.stop()?;
        },

        AgentSubcommand::Run => {
            Agent::new().run().await?;
        },

        AgentSubcommand::RunWindowsService => {
            tokio::spawn(async {
                Agent::new().run().await?;
                Ok::<(), anyhow::Error>(())
            });
            service_dispatcher::start(&Agent::SERVICE_NAME, ffi_service_main)?;
        },
    }

    Ok(())
}

async fn agent_status() -> anyhow::Result<()> {
    let agent_service_manager = SystemService::new(Agent::SERVICE_NAME.into());

    let service_status = agent_service_manager.status()?;
    match service_status {
        ServiceStatus::Uninstalled => println!("Porcelet agent service is not installed."),
        ServiceStatus::Stopped => println!("Porcelet agent service is not running."),
        ServiceStatus::Running => {},
    }

    // Query the service even if the service manager states it is not running,
    // for testing purposes, but don't report an error unless it expected to
    // be running.
    match Agent::query_status().await {
        Ok(status) => {
            if service_status != ServiceStatus::Running {
                log::warn!("Agent is running outside of the system service manager, this should only happen in testing");
            }
            println!("  Counter: {}", status);
            Ok(())
        },
        Err(err) =>  {
            if service_status == ServiceStatus::Running {
                Err(err)
            } else {
                Ok(())
            }
        },
    }
}

/// Porcelet CLI entry point.
/// 
/// If args is None, args are parsed from the command line.
pub async fn cli_main(args: Option<CliArgs>) -> ! {
    let args = args.unwrap_or(CliArgs::parse());

    let result = match args.subcommand {
        CliSubcommand::Agent { agent_subcommand } => agent_command(agent_subcommand).await,
        CliSubcommand::Status => agent_status().await,
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(err) => {
            log::error!("Error: {}", err);
            std::process::exit(1)
        },
    }
}
