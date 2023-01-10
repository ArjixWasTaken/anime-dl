#![allow(non_upper_case_globals)]

use reqwest::blocking::Client;

use crate::types::{AnimeEpisode, SearchResult};
mod animeonsen;

macro_rules! provider_call {
    ($provider_name:expr, $method:ident, $args:expr) => {
        match $provider_name {
            "animeonsen" => Some(animeonsen::$method($args)),
            _ => None,
        }
    };
}

pub fn search(
    client: &Client,
    provider_name: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let Some(ref result) = provider_call!(provider_name, search, (client, &query)) else {
        return Err("Provider not found".into());
    };

    return Ok(result.clone());
}

pub fn get_episodes(
    client: &Client,
    provider_name: &str,
    anime_url: &str,
) -> Result<Vec<AnimeEpisode>, String> {
    let Some(ref result) = provider_call!(provider_name, get_episodes, (client, &anime_url)) else {
        return Err("Provider not found".into());
    };

    return Ok(result.clone());
}
