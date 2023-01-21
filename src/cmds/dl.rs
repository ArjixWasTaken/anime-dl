use anyhow::{anyhow, bail, Result};
use clap::ArgMatches;
use reqwest_middleware::ClientWithMiddleware;

use crate::providers;
use crate::types::SearchResult;
use crate::utils::search_results_to_table;

pub async fn command(client: &ClientWithMiddleware, args: &ArgMatches<'_>) -> Result<()> {
    let provider = args.value_of("provider").unwrap();
    let choice = args
        .value_of("choice")
        .unwrap_or("-1")
        .parse::<i32>()
        .unwrap();
    let query = args.value_of("query").unwrap();
    let ep_range = args.value_of("episode").unwrap_or("1:");

    let search_results = providers::search(client, provider, query).await?;

    if search_results.is_empty() {
        crate::terminal::error(
            "No anime was found using that query, try again with another provider or keyword!",
        );
        bail!("NoSearchResults");
    }

    let mut chosen: &SearchResult;

    if choice < 1 {
        println!("{}", search_results_to_table(&search_results).to_string());
        let mut input = "".to_string();
        let mut first_loop = true;

        loop {
            if input.trim().len() >= 1 && input.trim().parse::<usize>().is_ok() {
                let idx = input.trim().parse::<usize>().unwrap();
                if idx > 0 && idx <= search_results.len() {
                    break;
                }
            }

            if !first_loop {
                println!(
                    "Please input a number within the range of 1-{}",
                    search_results.len()
                );
            }

            first_loop = false;
            input = casual::prompt("Select an anime [1]: ")
                .default("1".to_string())
                .get();
        }

        chosen = search_results
            .get(input.trim().parse::<usize>().unwrap() - 1)
            .unwrap();
    } else {
        if choice > search_results.len() as i32 {
            crate::terminal::error(format!("--choice/-c with the value of {}, is more than the number of the available search results ({})", choice, search_results.len()));
            bail!("InvalidChoice");
        }
        chosen = search_results.get((choice - 1) as usize).unwrap();
    }

    let episodes = providers::get_episodes(client, provider, chosen.url.as_str());
    let episodes = episodes.await?;

    let ep_range = crate::utils::parse_episode_range(
        ep_range,
        episodes.iter().map(|x| x.ep_num).max().unwrap_or(1),
    );

    let episodes = episodes
        .iter()
        .filter(|ep| ep_range.contains(&ep.ep_num))
        .collect::<Vec<_>>();

    let not_found_episodes = ep_range
        .iter()
        .filter(|ep_num| !episodes.iter().any(|ep| ep.ep_num == **ep_num))
        .collect::<Vec<_>>();

    if episodes.is_empty() {
        crate::terminal::error(format!(
            "Couldn't find any of the queried episodes! ({})",
            not_found_episodes
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        bail!("NoEpisodesFound");
    } else if !not_found_episodes.is_empty() {
        crate::terminal::error(format!(
            "Couldn't find the following episodes: {:?}",
            not_found_episodes
        ));
    }
    println!("Episodes chosen: {:#?}", episodes);

    Ok(())
}
