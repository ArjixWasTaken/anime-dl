use anyhow::{anyhow, bail, Result};
use clap::ArgMatches;
use reqwest_middleware::ClientWithMiddleware;

use crate::m3u8::M3u8Stream;
use crate::providers;
use crate::types::SearchResult;
use crate::utils::search_results_to_table;

pub async fn command(
    config: &crate::config::Config,
    client: &ClientWithMiddleware,
    args: &ArgMatches<'_>,
) -> Result<()> {
    let provider = args.value_of("provider").unwrap();
    let choice = args
        .value_of("choice")
        .unwrap_or("-1")
        .parse::<i32>()
        .unwrap();
    let query = args.value_of("query").unwrap();
    let ep_range = args.value_of("episodes").unwrap_or("1:");

    let search_results = providers::search(client, provider, query).await?;

    let chosen = crate::utils::user_select_result(search_results, choice)?;

    let episodes = providers::get_episodes(client, provider, chosen.url.as_str());
    let episodes = episodes.await?;

    let latest_ep = episodes.iter().map(|x| x.ep_num).max().unwrap_or(1);
    let mut ep_range = crate::utils::parse_episode_range(ep_range, latest_ep.clone());

    if args.is_present("last-episode") {
        ep_range = vec![latest_ep];
    }

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
    crate::terminal::debug(format!("Episodes chosen: {:?}", episodes));

    let single_ep = episodes.first().unwrap();
    let streams = crate::providers::get_streams(client, provider, &single_ep.url).await;

    crate::terminal::debug(format!("Streams for '{}': {:?}", single_ep.url, streams));

    let stream = streams?
        .0
        .iter()
        .find(|x| x.title == "HLS")
        .unwrap()
        .clone();

    let m3u8_stream = M3u8Stream {
        url: stream.url,
        quality: None,
        resolution: None,
        headers: stream.headers,
    };

    let m3u8_qualities = crate::m3u8::extract_streams(client, m3u8_stream, None).await;
    dbg!(m3u8_qualities);

    Ok(())
}
