use std::{collections::HashMap, ops::Index};

use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

use crate::types::{AnimeEpisode, SearchResult, StreamLink};

const host: &str = "animeonsen.xyz";

#[rustfmt::skip]
const search_authentication_header: &str = "Bearer 0e36d0275d16b40d7cf153634df78bc229320d073f565db2aaf6d027e0c30b13";
const api_authentication_header: &str = "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImRlZmF1bHQifQ.eyJpc3MiOiJodHRwczovL2F1dGguYW5pbWVvbnNlbi54eXovIiwiYXVkIjoiaHR0cHM6Ly9hcGkuYW5pbWVvbnNlbi54eXoiLCJpYXQiOjE2NzI5NjY2NjAsImV4cCI6MTY3MzU3MTQ2MCwic3ViIjoiMDZkMjJiOTYtNjNlNy00NmE5LTgwZmMtZGM0NDFkNDFjMDM4LmNsaWVudCIsImF6cCI6IjA2ZDIyYjk2LTYzZTctNDZhOS04MGZjLWRjNDQxZDQxYzAzOCIsImd0eSI6ImNsaWVudF9jcmVkZW50aWFscyJ9.VH_BpA8HMr5-zymN-zDZJO6cI48D73gdJZ3vT5Iu5E_raCozl5JC3oBgnv84PYZO_lturlc2xrtC425_cSldUGGUF48LtcB5P29y0ryCtoAXamdEXctBIPuAQdJFYFANbaOK4WMTsPa1WZax0yiFPV_9DxurGwoOTiNKXUuqkPC0wLdD51zRoa0wizm9iaancbjKkr0xWqWxRGReVM9lLMP8leGVm6VSNqiRfnSiqphT7Zxur7S3oRju4GmeW7QiY1XRZmz8VhRtmxiFU5S65WGLpi8k84KfUZ44TLd3D9bbaOkg9lRTxm49YzsaFpGJxpaq9LOp3Iol43sekLj8hw";

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

pub type EpisodeNumber = String;
pub type EpisodesResponse = HashMap<EpisodeNumber, Episode>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    #[serde(rename = "contentTitle_episode_en")]
    pub content_title_episode_en: Option<String>,
    #[serde(rename = "contentTitle_episode_jp")]
    pub content_title_episode_jp: Option<String>,
}

pub async fn search(args: (&ClientWithMiddleware, &str)) -> Option<Vec<SearchResult>> {
    let (client, query) = args;

    let mut json = HashMap::new();
    json.insert("q", query);

    let res = client
        .post(format!("https://search.{}/indexes/content/search", host))
        .header(AUTHORIZATION, search_authentication_header)
        .json(&json)
        .send().await.ok()? else {
            println!("Errored!");
            return None;
        };

    let json = res.json::<SearchReponse>().await.ok()? else {
            println!("Errored!");
            return None;
        };

    if json.hits.is_none() {
        return None;
    }

    Some(
        json.clone()
            .hits
            .unwrap()
            .iter()
            .map(|hit| SearchResult {
                title: hit.content_title.clone().unwrap(),
                url: format!(
                    "https://{}/details/{}",
                    host,
                    hit.content_id.clone().unwrap()
                ),
                provider: "animeonsen".to_string(),
            })
            .collect::<Vec<SearchResult>>(),
    )
}

pub async fn get_episodes(args: (&ClientWithMiddleware, &str)) -> Option<Vec<AnimeEpisode>> {
    let (client, url) = args;
    let Ok(ref id_regex) = regex::Regex::new(r#"animeonsen.xyz/details/(.+)/?"#) else {
        return None;
    };

    let Some(ref id) = id_regex.captures(url) else {
        return None;
    };

    let res = client
        .get(format!(
            "https://api.{}/v4/content/{}/episodes",
            host,
            id.index(1)
        ))
        .header(AUTHORIZATION, api_authentication_header)
        .send()
        .await
        .ok()?;

    let json = res.json::<EpisodesResponse>().await.ok()?;
    let mut episodes = Vec::new();

    for (episode_number, episode) in json {
        let ep_num = episode_number.parse::<i32>().unwrap_or(-1);
        let title = episode.content_title_episode_en.clone().unwrap_or_else(|| {
            episode
                .content_title_episode_jp
                .clone()
                .unwrap_or_else(|| "".to_string())
        });

        episodes.push(AnimeEpisode {
            ep_num,
            title,
            url: format!(
                "https://{}/watch/{}?episode={}",
                host,
                id.index(1),
                episode_number
            ),
            provider: "animeonsen".to_string(),
        });
    }

    episodes.sort_by(|a, b| a.ep_num.partial_cmp(&b.ep_num).unwrap());

    Some(episodes)
}

pub async fn get_streams(args: (&ClientWithMiddleware, &str)) -> Option<Vec<StreamLink>> {
    let (client, url) = args;

    unreachable!("Not implemented yet, lol");
}
