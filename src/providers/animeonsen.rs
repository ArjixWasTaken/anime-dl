use std::collections::HashMap;

use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use crate::types::{AnimeEpisode, SearchResult};

const base_url: &str = "https://animeonsen.xyz";
const authentication_header: &str =
    "Bearer 0e36d0275d16b40d7cf153634df78bc229320d073f565db2aaf6d027e0c30b13";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchReponse {
    pub hits: Option<Vec<Hit>>,
    #[serde(rename = "estimatedTotalHits")]
    pub estimated_total_hits: Option<i64>,
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    #[serde(rename = "processingTimeMs")]
    pub processing_time_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hit {
    pub content_title: Option<String>,
    pub content_title_en: Option<String>,
    pub content_title_jp: Option<String>,
    pub content_id: Option<String>,
}

pub fn search(args: (&Client, &str)) -> Vec<SearchResult> {
    let (client, query) = args;

    let mut json = HashMap::new();
    json.insert("q", query);

    let Ok(res) = client
        .post("https://search.animeonsen.xyz/indexes/content/search")
        .header(AUTHORIZATION, authentication_header)
        .json(&json)
        .send() else {
            println!("Errored!");
            return Vec::new();
        };

    let Ok(ref json) = res.json::<SearchReponse>() else {
            println!("Errored!");
            return Vec::new();
        };

    if json.hits.is_none() {
        return Vec::new();
    }

    json.clone()
        .hits
        .unwrap()
        .iter()
        .map(|hit| SearchResult {
            title: hit.content_title.clone().unwrap(),
            url: format!("{}/details/{}", base_url, hit.content_id.clone().unwrap()),
            provider: "animeonsen".to_string(),
        })
        .collect::<Vec<SearchResult>>()
}

pub fn get_episodes(args: (&Client, &str)) -> Vec<AnimeEpisode> {
    let (client, url) = args;

    let Some(ref id) = regex::Regex::new(r#"animeonsen.xyz/details/(.+)/?"#).unwrap().captures(url) else {
        return Vec::new();
    };

    println!("id! {:#?}", id);

    return vec![];
}
