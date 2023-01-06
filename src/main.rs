#![allow(warnings)]
#[macro_use]
extern crate clap;
extern crate term;

mod cli;
mod cmds;
mod errors;
mod providers;
mod terminal;
mod types;

use crate::cmds::dl;

fn main() {
    let matches = cli::build_cli().get_matches();

    if let Some(args) = matches.subcommand_matches("dl") {
        dl::command(args);
    }
}
