use anyhow::{anyhow, bail, Result};
use clap::ArgMatches;
use reqwest_middleware::ClientWithMiddleware;

use crate::providers;
use crate::types::{SearchResult, StreamLink};
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
    let chosen = crate::utils::user_select_result(search_results, choice)?;

    let episodes = providers::get_episodes(client, provider, chosen.url.as_str()).await?;

    let ep_range = crate::utils::parse_episode_range(
        ep_range,
        episodes.iter().map(|x| x.ep_num).max().unwrap_or(1),
    );

    let episodes = episodes
        .iter()
        .filter(|x| ep_range.contains(&x.ep_num))
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
        return Ok(());
    } else if !not_found_episodes.is_empty() {
        crate::terminal::error(format!(
            "Couldn't find the following episodes: {:?}",
            not_found_episodes
        ));
    }

    for episode in episodes {
        let Ok((streams, subtitles)) = providers::get_streams(client, provider, episode.url.as_str()).await else {
            continue;
        };

        let streams = crate::extractors::unpack_streams(client, streams).await;
        println!("Streams for episode {}: {:#?}", episode.ep_num, streams);
    }

    Ok(())
}
