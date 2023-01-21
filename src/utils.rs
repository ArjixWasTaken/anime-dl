use anyhow::{bail, Result};
use comfy_table::presets::ASCII_FULL;
use comfy_table::Table;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use std::collections::HashSet;
use std::hash::Hash;

use crate::types::{AnimeEpisode, SearchResult};

pub fn search_results_to_table(search_results: &Vec<SearchResult>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_header(vec!["SlNo", "Title", "Provider"]);

    for (i, result) in search_results.iter().enumerate() {
        table.add_row(vec![
            (i + 1).clone().to_string(),
            result.title.clone(),
            result.provider.clone(),
        ]);
    }
    return table;
}

pub fn user_select_result(search_results: Vec<SearchResult>, choice: i32) -> Result<SearchResult> {
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

    Ok(chosen.clone())
}

pub fn parse_episode_range(episode_range: &str, latest_ep: i32) -> Vec<i32> {
    let mut episodes: Vec<i32> = Vec::new();

    for ep_range in episode_range.split(",") {
        let range = ep_range
            .trim()
            .split(":")
            .map(|x| x.trim())
            .collect::<Vec<&str>>();

        match range.len() {
            1 => {
                episodes.push(range[0].parse().unwrap());
            }
            2 => {
                if range[0].is_empty() && range[1].is_empty() {
                    crate::terminal::error("Both start and end of range are empty!");
                    continue;
                }

                if range[0].is_empty() {
                    let end = range[1].parse::<i32>().unwrap();
                    for i in 1..=end {
                        episodes.push(i);
                    }
                } else if range[1].is_empty() {
                    let start = range[0].parse::<i32>().unwrap();

                    for i in start..=latest_ep {
                        episodes.push(i);
                    }
                } else {
                    let start = range[0].parse::<i32>().unwrap();
                    let end = range[1].parse::<i32>().unwrap();
                    for i in start..=end {
                        episodes.push(i);
                    }
                }
            }
            _ => {
                crate::terminal::error(format!("{:?} is not a valid episode range!", range));
                continue;
            }
        }
    }

    dedup(&mut episodes);
    episodes.sort();

    episodes
}

pub fn dedup<T: Eq + Hash + Copy>(v: &mut Vec<T>) {
    // note the Copy constraint
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(*e));
}

pub async fn download_episodes(
    client: &ClientWithMiddleware,
    episodes: Vec<&AnimeEpisode>,
) -> Result<bool> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Origin",
        header::HeaderValue::from_str("https://www.animeonsen.xyz/")?,
    );
    headers.insert(
        "Referer",
        header::HeaderValue::from_str("https://www.animeonsen.xyz/")?,
    );

    for episode in episodes {
        let streams =
            crate::providers::get_streams(client, &episode.provider, episode.url.as_str())
                .await
                .unwrap();

        println!("Streams! {:#?}", streams);
        break;
    }

    Ok(true)
}
