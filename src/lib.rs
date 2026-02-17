// SPDX-FileCopyrightText: 2026 (c) Gary "BLumia" Wang <opensource@blumia.net>
//
// SPDX-License-Identifier: MIT

pub mod cli;
pub mod create;
pub mod extract;
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
            run(parsed)
        }
    }
}

fn run(args: types::ParsedArgs) -> i32 {
    match args.command {
        Command::List => list::run_list(&args),
        Command::Create => create::run_create(&args),
        Command::Extract => extract::run_extract(&args),
    }
}
