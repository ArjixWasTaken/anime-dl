#![allow(warnings)]
#[macro_use]
extern crate clap;
extern crate anyhow;
extern crate casual;
extern crate comfy_table;
extern crate http_cache_reqwest;
extern crate reqwest;
extern crate reqwest_middleware;
extern crate reqwest_retry;
extern crate serde;
extern crate serde_json;
extern crate term;
extern crate term_painter;

mod cli;
mod cmds;
mod providers;
mod terminal;
mod types;
mod utils;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

macro_rules! try_cmd {
    ($cmd:ident, $matches:ident, $client:ident) => {
        if let Some(args) = $matches.subcommand_matches(stringify!($cmd)) {
            let cmd_str = stringify!($cmd);
            crate::terminal::debug(format!("Executing the '{}' subcommand.", cmd_str));
            crate::cmds::$cmd::command(&$client, args).await.unwrap();
            crate::terminal::debug(format!(
                "Finished the execution of the '{}' subcommand.",
                cmd_str
            ));
            return;
        }
    };
}

#[tokio::main]
async fn main() {
    let mut app = cli::build_cli();
    let matches = app.clone().get_matches();

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::default(),
            options: None,
        }))
        .build();

    unsafe {
        crate::terminal::VERBOSITY = matches.occurrences_of("verbose");
    }

    try_cmd!(dl, matches, client);
    try_cmd!(watch, matches, client);

    // If no subcommand was matched, print the help message
    app.print_help();
    println!(); // clap does not add a newline at the end for some reason...
}
