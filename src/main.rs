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
mod extractors;
mod providers;
mod terminal;
mod types;
mod utils;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::header::USER_AGENT;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

macro_rules! exec_cmd {
    ($cmd:ident, $client:expr, $args:expr) => {
        {
            let cmd_str = stringify!($cmd).replace("_", "");
            crate::terminal::debug(format!("Executing the '{}' subcommand.", cmd_str));
            match crate::cmds::$cmd::command(&$client, $args).await {
                Ok(_) => (),
                Err(error) => match error.to_string().as_str() {
                    "NoEpisodesFound" | "NoSearchResults" => (/* The error is known and has already been logged. */),
                    _ => {
                        // The error is unknown, and should be logged.
                        crate::terminal::error(format!("Unhandled error: {}", error));
                        return;
                    }
                }
            }
            crate::terminal::debug(format!(
                "Finished the execution of the '{}' subcommand.",
                cmd_str
            ));
        };
    }
}

#[tokio::main]
async fn main() {
    ctrlc::set_handler(|| {
        spinach::term::show_cursor();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut app = cli::build_cli();
    let matches = app.clone().get_matches();

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(
        reqwest::ClientBuilder::new()
            .default_headers( reqwest::header::HeaderMap::from_iter(vec![(
                USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36".parse().unwrap()
            )])
        )
        .build().unwrap()
    )
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

    match matches.subcommand() {
        ("dl", Some(args)) => exec_cmd!(dl, client, args),
        ("watch", Some(args)) => exec_cmd!(watch, client, args),
        ("self", Some(args)) => exec_cmd!(self_, client, args),
        _ => (),
    }
}
