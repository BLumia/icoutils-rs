pub mod cli;
pub mod input;
pub mod list;
pub mod parse;
pub mod types;
pub mod util;

use types::{Action, Command};

pub fn run_from_args(program_path: &str, argv: &[String]) -> i32 {
    let program_name = util::program_basename(program_path);

    let (action, parsed) = match cli::parse_args(argv) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{msg}");
            return 1;
        }
    };

    match action {
        Action::Help => {
            cli::print_help(&program_name);
            0
        }
        Action::Version => {
            cli::print_version(&program_name);
            0
        }
        Action::Run => {
            let Some(parsed) = parsed else {
                eprintln!("missing argument");
                cli::print_help(&program_name);
                return 1;
            };
            run(program_name, parsed)
        }
    }
}

fn run(program_name: String, args: types::ParsedArgs) -> i32 {
    match args.command {
        Command::List => list::run_list(&args),
        Command::Extract | Command::Create => {
            eprintln!("{program_name}: not implemented yet");
            1
        }
    }
}
