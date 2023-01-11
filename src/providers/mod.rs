#![allow(non_upper_case_globals)]

use crate::types::{AnimeEpisode, SearchResult};
use reqwest::Client;
mod animeonsen;
use reqwest_middleware::ClientWithMiddleware;

macro_rules! provider_call {
    ($provider_name:expr, $method:ident, $args:expr) => {
        match $provider_name {
            "animeonsen" => Some(animeonsen::$method($args).await?),
            _ => None,
        }
    };
}

pub async fn search(
    client: &ClientWithMiddleware,
    provider_name: &str,
    query: &str,
) -> Option<Vec<SearchResult>> {
    let Some(ref result) = provider_call!(provider_name, search, (client, &query)) else {
        crate::terminal::error(format!("Provider '{}' not found", provider_name).as_str());
        return None;
    };

    crate::terminal::success(format!("Executed '{}::search()'", provider_name).as_str());
    return Some(result.clone());
}

pub async fn get_episodes(
    client: &ClientWithMiddleware,
    provider_name: &str,
    anime_url: &str,
) -> Option<Vec<AnimeEpisode>> {
    let Some(ref result) = provider_call!(provider_name, get_episodes, (client, &anime_url)) else {
        crate::terminal::error(format!("Provider '{}' not found", provider_name).as_str());
        return None;
    };

    crate::terminal::success(format!("Executed '{}::get_episodes()'", provider_name).as_str());
    return Some(result.clone());
}
