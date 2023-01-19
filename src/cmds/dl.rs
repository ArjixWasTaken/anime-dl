use clap::ArgMatches;
use reqwest_middleware::ClientWithMiddleware;

use crate::providers;
use crate::types::SearchResult;
use crate::utils::search_results_to_table;

pub async fn command(client: &ClientWithMiddleware, args: &ArgMatches<'_>) -> i16 {
    let provider = args.value_of("provider").unwrap();
    let choice = args
        .value_of("choice")
        .unwrap_or("-1")
        .parse::<i32>()
        .unwrap();
    let query = args.value_of("query").unwrap();
    let ep_range = args.value_of("episode").unwrap_or("1:");

    let Some(search_results) = providers::search(client, provider, query).await else {
        return 1; // Error
    };

    if search_results.is_empty() {
        crate::terminal::error(
            "No anime was found using that query, try again with another provider or keyword!",
        );
        return 1; // Error
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
            return 1; // Error
        }
        chosen = search_results.get((choice - 1) as usize).unwrap();
    }

    let episodes = providers::get_episodes(client, provider, chosen.url.as_str());
    let Some(episodes) = episodes.await else {
        crate::terminal::error("Failed to get episodes!");
        return 1; // Error
    };

    let ep_range = crate::utils::parse_episode_range(
        ep_range,
        episodes.iter().map(|x| x.ep_num).max().unwrap_or(1),
    );

    let Ok(ep_range) = ep_range else {
        crate::terminal::error("Could not parse --episode/-e");
        return 1;
    };

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
        return 1;
    } else if !not_found_episodes.is_empty() {
        crate::terminal::error(format!(
            "Couldn't find the following episodes: {:?}",
            not_found_episodes
        ));
    }

    // crate::terminal::info();
    println!("Episodes chosen: {:#?}", episodes);

    return 0; // Ok
}
