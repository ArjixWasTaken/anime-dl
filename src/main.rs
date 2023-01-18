#![allow(warnings)]
#[macro_use]
extern crate clap;
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
mod errors;
mod providers;
mod terminal;
mod types;
mod utils;

use crate::cmds::dl;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

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

    if let Some(args) = matches.subcommand_matches("dl") {
        terminal::info("Executing the 'dl' subcommand.");
        dl::command(&client, args).await;
        terminal::info("Finished the execution of the 'dl' subcommand.");
    } else {
        app.print_help();
        println!(""); // clap does not add a newline at the end for some reason...
    }
}
