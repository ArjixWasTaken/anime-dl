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

    for provider in crate::cli::PROVIDERS {
        s.text(format!(" Testing {}", provider));

        // Note: this doesnt need to be async, but changing it would require rewriting the macro...
        let url = crate::providers::get_test_url(client, provider, "0").await?;
        let episodes = crate::providers::test_episodes(client, provider, &url).await;

        let mut check = "✖";
        let mut color = Color::Red;
        let mut details: String = String::new();

        match episodes {
            Ok((found, expected)) => {
                if found == expected {
                    check = "✔";
                    color = Color::Green;
                }

                let sfound = {
                    let s = found.to_string();
                    let lpad = expected.to_string().len() - s.len();
                    " ".repeat(lpad).to_string() + &s
                };

                details = format!(
                    "{}[ {} out of {} eps ]",
                    " ".repeat(padding - provider.len() + 1),
                    Plain.fg(term_painter::Color::Green).paint(sfound),
                    Plain.fg(term_painter::Color::Green).paint(expected)
                );
            }
            Err(err) => {
                println!("{:#?}", err);
            }
        }

        s.freeze(check, format!(" {}{}", provider, details), color, None);
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
