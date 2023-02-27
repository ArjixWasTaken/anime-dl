use crate::types::SearchResult;
use anyhow::{anyhow, bail, Result};
use clap::{ArgMatches, SubCommand};
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use serde_aux::serde_introspection;
use spinach::{Color, Spinach};
use std::any::TypeId;
use term_painter::{
    Attr::Plain,
    Color::{Cyan, Yellow},
    ToStyle,
};
use difference::{Difference, Changeset};


pub async fn update(config: &crate::config::Config, args: &ArgMatches<'_>) -> Result<()> {
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
            _ => (),
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

    for provider in crate::cli::PROVIDERS {
        s.text(format!(" Testing {}", provider));

        // Note: this doesnt need to be async, but changing it would require rewriting the macro...
        let url = crate::providers::get_test_url(client, provider, "1").await?;
        let streams = crate::providers::test_streams(client, provider, &url).await;

        let mut check = "✖";
        let mut color = Color::Red;
        let mut details: String = String::new();

        match streams {
            Ok(found) => {
                if found > 0 {
                    check = "✔";
                    color = Color::Green;
                }

                details = format!(
                    "{}[ found {} stream{} ]",
                    " ".repeat(padding - provider.len() + 1),
                    Plain.fg(term_painter::Color::Green).paint(found),
                    if found != 1 { "s" } else { " " }
                );
            }
            _ => (),
        }

        s.freeze(check, format!(" {}{}", provider, details), color, None);
        s.text(" Testing ...");
    }
    s.stop_with("", "", None);

    Ok(())
}

pub async fn test(
    client: &ClientWithMiddleware,
    config: &crate::config::Config,
    args: &ArgMatches<'_>,
) -> Result<()> {
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

pub async fn config_(
    client: &ClientWithMiddleware,
    config: &crate::config::Config,
    args: &ArgMatches<'_>,
) -> Result<()> {
    // !! Temporary code !!
    // [@ArjixWasTaken] Hey @Amanse,
    // I won't tell you much, but I want this to be generic,
    // use serde to serialise and deserialise the config,
    // that way you can iterate over all the struct fields
    // I don't want a config command that needs updating whenever the config changes, if you get what I'm saying.
    //
    //
    let fields = serde_introspection::serde_introspect::<crate::config::Config>();

    let selection = dialoguer::Select::new()
        .items(&fields)
        .default(0)
        .interact_opt()
        .unwrap()
        .unwrap();

    let new_val: String = {
        match config.get(fields[selection]) {
            Ok(v) =>  {
                if v == TypeId::of::<bool>() {
                    let c = dialoguer::Confirm::new().with_prompt(fields[selection].clone()).interact().unwrap();
                    if c {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                } else  {
                    dialoguer::Input::new().with_prompt(fields[selection].clone()).interact_text().unwrap()
                } 
            },
            Err(_) => {
                panic!("bithc");
            }
        }
    };

    let new = config.clone().update(fields[selection], new_val.as_str())?;

    show_diff(serde_yaml::to_string(&config)?, serde_yaml::to_string(&new)?);

    println!(
        "{}\n{}",
        Plain
            .fg(term_painter::Color::BrightCyan)
            .paint("Current config:"),
        Plain
            .fg(term_painter::Color::BrightGreen)
            .paint(serde_yaml::to_string(&config)?),
    );

    Ok(())
}

fn type_id<T: 'static + ?Sized>(_: &T) -> TypeId {
    TypeId::of::<T>()
}

fn show_diff(text1: String, text2: String) {
    let Changeset { diffs, .. } = Changeset::new(text1.as_str(), text2.as_str(), "\n");

    let mut t = term::stdout().unwrap();

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => {
                t.reset().unwrap();
                writeln!(t, " {}", x);
            }
            Difference::Add(ref x) => {
                t.fg(term::color::GREEN).unwrap();
                writeln!(t, "+{}", x);
            }
            Difference::Rem(ref x) => {
                t.fg(term::color::RED).unwrap();
                writeln!(t, "-{}", x);
            }
        }
    }
    t.reset().unwrap();
    t.flush().unwrap();
}

pub async fn command(
    config: &crate::config::Config,
    client: &ClientWithMiddleware,
    args: &ArgMatches<'_>,
) -> Result<()> {
    match args.subcommand() {
        ("update", Some(sub_args)) => update(config, sub_args).await?,
        ("test", Some(sub_args)) => test(client, config, sub_args).await?,
        ("config", Some(sub_args)) => config_(client, config, sub_args).await?,
        _ => (),
    }

    Ok(())
}
