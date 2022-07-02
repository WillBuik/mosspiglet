use std::result;

use tokio::process::{Command, ChildStdin};

mod cli;

struct CommandOptions {
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
}

#[tokio::main]
async fn main() {
    let result = cli::cli_main(None).await;

    match result {
        Ok(result) => {
            if !result.success() {
                log::error!("Error: {}", result)
            }
            std::process::exit(result.status());
        },
        Err(err) => {
            log::error!("Unknwon error: {}", err);
            std::process::exit(cli::PorceletCliStatus::UnknownError.status());
        },
    }
}
