use anyhow::{anyhow, bail, Result};
use clap::{ArgMatches, SubCommand};
use reqwest_middleware::ClientWithMiddleware;
use term_painter::{
    Attr::Plain,
    Color::{Cyan, Yellow},
    ToStyle,
};

use spinach::{Color, Spinach};
use std::ops::Not;

use crate::types::SearchResult;

pub async fn update(args: &ArgMatches<'_>) -> Result<()> {
    // TODO: Use the jaemk/self_update crate to implement this.
    bail!("Whine about it, lul.")
}

pub async fn test_search(client: &ClientWithMiddleware, args: &ArgMatches<'_>) -> Result<()> {
    // Defaults to "Overlord"
    let query = args.value_of("query").unwrap();

    Plain.bold().with(|| {
        println!("Searching `{}`:", Plain.fg(Cyan).paint(query));
    });

    let padding = crate::cli::PROVIDERS.iter().map(|x| x.len()).max().unwrap();

    let s = Spinach::new(" Testing ...");
    s.color(Color::Green);

    for provider in crate::cli::PROVIDERS {
        s.text(format!(" Testing {}", provider));

        let search = crate::providers::search(client, provider, query).await;
        if search.is_ok() {
            let num = search.unwrap().len();
            if num != 0 {
                let snum = num.to_string();
                s.freeze(
                    "✔",
                    format!(
                        " {}{}[ {} search results ]",
                        provider,
                        " ".repeat((padding - &provider.len() + 1)),
                        if snum.len() < 2 {
                            " ".to_string() + &snum
                        } else {
                            snum
                        },
                    ),
                    None,
                    None,
                );
            } else {
                s.freeze("✖", format!(" {} [ 0 search results ]", provider), Color::Red, None);
            }
        } else {
            s.freeze("✖", format!(" {}", provider), Color::Red, None);
        }
        s.text(" Testing ...");
    }
    s.stop_with("", "", None);

    Ok(())
}

pub async fn test_episodes(client: &ClientWithMiddleware) -> Result<()> {
    println!("{}", Plain.bold().paint("Fetching episodes:"));

    let padding = crate::cli::PROVIDERS.iter().map(|x| x.len()).max().unwrap();

    let s = Spinach::new(" Testing ...");
    s.color(Color::Green);

    // TODO: Actually implement this...

    for provider in crate::cli::PROVIDERS {
        s.text(format!(" Testing {}", provider));

        let search = crate::providers::search(client, provider, "overlord").await;
        let mut check = "✖";

        if search.is_ok() {
            let val = search.unwrap();
            let result = val.first();

            if result.is_none().not() {
                check = "✔";
                let url = &result.unwrap().url;
                let episodes = crate::providers::get_episodes(client, provider, url).await;

                if episodes.is_ok() {
                    let val = episodes.unwrap();
                    let eps = val.first();

                    if eps.is_none() {
                        check = "✖";
                    }
                } else {
                    check = "✖";
                }
            }
        }

        s.freeze(
            check,
            format!(" {}", provider,),
            if check.eq("✔") {
                Color::Green
            } else {
                Color::Red
            },
            None,
        );
        s.text(" Testing ...");
    }
    s.stop_with("", "", None);

    Ok(())
}

pub async fn test_streams(client: &ClientWithMiddleware) -> Result<()> {
    println!("{}", Plain.bold().paint("Fetching streams:"));

    let padding = crate::cli::PROVIDERS.iter().map(|x| x.len()).max().unwrap();

    let s = Spinach::new(" Testing ...");
    s.color(Color::Green);

    // TODO: Actually implement this...

    for provider in crate::cli::PROVIDERS {
        s.text(format!(" Testing {}", provider));

        let search = crate::providers::search(client, provider, "overlord").await;
        s.freeze(
            if search.is_ok() { "✔" } else { "✖" },
            format!(" {}", provider,),
            if search.is_ok() {
                Color::Green
            } else {
                Color::Red
            },
            None,
        );
        s.text(" Testing ...");
    }
    s.stop_with("", "", None);

    Ok(())
}

pub async fn test(client: &ClientWithMiddleware, args: &ArgMatches<'_>) -> Result<()> {
    println!(
        "\nTesting {} the providers...\n",
        Cyan.paint(crate::cli::PROVIDERS.len()),
    );
    test_search(client, args).await?;
    println!();
    test_episodes(client).await?;
    println!();
    test_streams(client).await?;

    Cyan.with(|| println!("Done!"));
    Ok(())
}

pub async fn command(client: &ClientWithMiddleware, args: &ArgMatches<'_>) -> Result<()> {
    match args.subcommand() {
        ("update", Some(sub_args)) => update(sub_args).await?,
        ("test", Some(sub_args)) => test(client, sub_args).await?,
        _ => (),
    }

    Ok(())
}
