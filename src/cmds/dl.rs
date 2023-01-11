use clap::ArgMatches;
use reqwest::Client;

use crate::providers;
use crate::types::SearchResult;
use crate::utils::search_results_to_table;

pub async fn command(client: &Client, args: &ArgMatches<'_>) -> i16 {
    let provider = args.value_of("provider").unwrap();
    let choice = args
        .value_of("choice")
        .unwrap_or("-1")
        .parse::<i32>()
        .unwrap();
    let query = args.value_of("query").unwrap();
    let ep_range = args.value_of("episode").unwrap_or("1:");

    crate::terminal::info(
        format!(
            "Attempting to search for '{}' using the '{}' provider!",
            query, provider
        )
        .as_str(),
    );

    let Some(search_results) = providers::search(client, provider, query).await else {
        return 1; // Error
    };

    let mut chosen: &SearchResult;

    if choice < 1 {
        println!("{}", search_results_to_table(&search_results).to_string());
        let mut input = "".to_string();

        loop {
            if input.trim().len() >= 1 && input.trim().parse::<usize>().is_ok() {
                let idx = input.trim().parse::<usize>().unwrap();
                if idx > 0 && idx <= search_results.len() {
                    break;
                }

                println!(
                    "Please input a number within the range of 1-{}",
                    search_results.len()
                );
            }

            input = casual::prompt("Select an anime [1]: ")
                .default("1".to_string())
                .get();
        }

        chosen = search_results
            .get(input.trim().parse::<usize>().unwrap() - 1)
            .unwrap();
    } else {
        if choice > search_results.len() as i32 {
            crate::terminal::error(format!("--choice/-c with the value of {}, is more than the number of the available search results ({})", choice, search_results.len()).as_str());
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

    println!("Episodes chosen: {:?}", ep_range.unwrap());
    return 0; // Ok
}
