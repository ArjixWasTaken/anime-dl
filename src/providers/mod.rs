#![allow(non_upper_case_globals)]

use reqwest::blocking::Client;

use crate::types::SearchResult;
mod animeonsen;

pub fn search(
    client: &Client,
    provider_name: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let Some(ref result) = (match provider_name {
        "animeonsen" => Some(animeonsen::search(client, &query)),
        _ => None,
    }) else {
        return Err("Provider not found.".into());
    };

    return Ok(result.clone());
}
