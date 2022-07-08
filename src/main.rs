use std::{ffi::OsString, time::Duration};

use agent::Agent;
use windows_service::{define_windows_service, service_control_handler::{self, ServiceControlHandlerResult}, service::{ServiceControl, ServiceType, ServiceState, ServiceControlAccept, ServiceExitCode}};

mod agent;
mod cli;
mod service;

/*struct CommandOptions {
    command: CommandLine,
    input: Option<Vec<u8>>,
}

impl CommandOptions {
    pub fn to_command(self) -> std::io::Result<Command> {
        let mut command = match self.command {
            CommandLine::Shell(shell) => {
                let mut args_itr = shell.split(" ");
                let program = args_itr.next();
                let args: Vec<String> = args_itr.map(|s| String::from(s)).collect();
                if let Some(program) = program {
                    let mut command = Command::new(program);
                    command.args(args);
                    command
                } else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty commandn line"));
                }
            },

            CommandLine::ProgramArgs { program, args } => {
                let mut command = Command::new(program);
                command.args(args);
                command
            },
        };

        if let Some(input) = self.input {
            //let input = ChildStdin::from(input);
        }
        
        Ok(command)
    }
}

enum CommandLine {
    Shell (String),
    ProgramArgs {
        program: String,
        args: Vec<String>,
    }
}

#[derive(Default)]
struct CommandResult {
    status: u32,
    output: Vec<u8>,
    output_err: Vec<u8>,
}

async fn run_command(command: CommandOptions) {
    //let proc = tokio::process::Command::new(program)
}*/

define_windows_service!(ffi_service_main, win_service_main);

fn win_service_main(_arguments: Vec<OsString>) {
    // The entry point where execution will start on a background thread after a call to
    // `service_dispatcher::start` from `main`.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Handle stop event and return control back to the system.
                // TODO: Actually stop the agent, getting a handle to it and awaiting
                //       the stop will probably require some unsafe nonsense :(
                ServiceControlHandlerResult::NoError
            }
            // All services must accept Interrogate even if it's a no-op.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register(Agent::SERVICE_NAME, event_handler);
    match status_handle {
        Ok(status_handle) => {
            let next_status = windows_service::service::ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            };
            if let Err(err) = status_handle.set_service_status(next_status) {
                log::error!("Failed to update service status to running: {}", err);
            }
        },

        Err(err) => {
            log::error!("Failed to register service control handler: {}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env().filter_level(log::LevelFilter::Info).init();
    cli::cli_main(None).await;
}
