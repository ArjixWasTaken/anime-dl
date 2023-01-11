#![allow(warnings)]
#[macro_use]
extern crate clap;
extern crate casual;
extern crate comfy_table;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate term;
extern crate term_painter;

mod cli;
mod cmds;
mod errors;
mod providers;
mod terminal;
mod types;
mod utils;

use crate::cmds::dl;

#[tokio::main]
async fn main() {
    let matches = cli::build_cli().get_matches();
    let client = reqwest::Client::new();

    unsafe {
        crate::terminal::VERBOSITY = matches.occurrences_of("verbose");
    }

    if let Some(args) = matches.subcommand_matches("dl") {
        terminal::info("Executing the 'dl' subcommand.");
        dl::command(&client, args).await;
        terminal::info("Finished the execution of the 'dl' subcommand.");
    }
}
