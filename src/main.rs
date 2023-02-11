#![allow(warnings)]
#[macro_use]
extern crate clap;
extern crate anyhow;
extern crate base64;
extern crate casual;
extern crate comfy_table;
extern crate cookie;
extern crate http_cache_reqwest;
extern crate reqwest;
extern crate reqwest_middleware;
extern crate reqwest_retry;
extern crate serde;
extern crate serde_json;
extern crate spinach;
extern crate term;
extern crate term_painter;
extern crate urlencoding;

mod cli;
mod cmds;
mod config;
mod extractors;
mod m3u8;
mod providers;
mod terminal;
mod types;
mod utils;

use anyhow::Result;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::header::USER_AGENT;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

macro_rules! exec_cmd {
    ($cmd:ident, $config:expr, $client:expr, $args:expr) => {
        {
            let cmd_str = stringify!($cmd).replace("_", "");
            crate::terminal::debug(format!("Executing the '{}' subcommand.", cmd_str));
            match crate::cmds::$cmd::command(&$config, &$client, $args).await {
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
    #[rustfmt::skip]
    ctrlc::set_handler(|| {
        spinach::term::show_cursor();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    let mut app = cli::build_cli();
    let matches = app.clone().get_matches();

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let mut client = ClientBuilder::new(
        reqwest::ClientBuilder::new()
            .default_headers( reqwest::header::HeaderMap::from_iter(vec![(
                USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36".parse().unwrap()
            )])
        )
        .build().unwrap()
    )
        .with(RetryTransientMiddleware::new_with_policy(retry_policy));

    if matches.subcommand_name() != Some("self") {
        // we don't want to use cache for self subcommands
        client = client.with(Cache(HttpCache {
            mode: CacheMode::ForceCache,
            manager: CACacheManager::default(),
            options: None,
        }));
    }

    let client = client.build();
    let mut config = crate::config::Config::load().expect("Failed to read/generate the config.");

    unsafe {
        match matches.occurrences_of("verbose") {
            0 => crate::terminal::VERBOSITY = config.verbosity,
            level => crate::terminal::VERBOSITY = level,
        }
    }

    match matches.subcommand() {
        ("dl", Some(args)) => exec_cmd!(dl, config, client, args),
        ("watch", Some(args)) => exec_cmd!(watch, config, client, args),
        ("self", Some(args)) => exec_cmd!(self_, config, client, args),
        ("print_config", Some(args)) => exec_cmd!(print_cfg, config, client, args),
        _ => (),
    }
}
