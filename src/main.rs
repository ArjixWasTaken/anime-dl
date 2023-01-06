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
    let app = cli::build_cli();
    let matches = app.get_matches();

    if let Some(args) = matches.subcommand_matches("dl") {
        dl::command(args);
    }
}
