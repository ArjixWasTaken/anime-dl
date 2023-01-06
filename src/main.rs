#![allow(warnings)]
#[macro_use]
extern crate clap;
extern crate casual;
extern crate comfy_table;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate term;

mod cli;
mod cmds;
mod errors;
mod providers;
mod terminal;
mod types;
mod utils;

use crate::cmds::dl;

fn main() {
    let matches = cli::build_cli().get_matches();
    let client = reqwest::blocking::Client::new();

    if let Some(args) = matches.subcommand_matches("dl") {
        dl::command(&client, args);
    }
}
