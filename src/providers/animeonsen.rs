use std::{collections::HashMap, ops::Index};

use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::types::{AnimeEpisode, SearchResult, StreamLink};

const host: &str = "animeonsen.xyz";

// python code on how to get the api token:
/*
import httpx
import base64
import urllib.parse

headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:108.0) Gecko/20100101 Firefox/108.0",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp",
    "Accept-Language": "en-US,en;q=0.5",
    "Upgrade-Insecure-Requests": "1",
    "Sec-Fetch-Dest": "document",
    "Sec-Fetch-Mode": "navigate",
    "Sec-Fetch-Site": "none",
    "Sec-Fetch-User": "?1",
}

res = httpx.get("https://www.animeonsen.xyz/", headers=headers)

cookie = urllib.parse.unquote(res.cookies.get("ao.session", ""))
authorization = "Bearer "
authorization += "".join(
    [chr(ord(x) + 1) for x in base64.b64decode(cookie).decode("utf-8")]
)

print(authorization)
*/

#[rustfmt::skip]
const api_authentication_header: &str = "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImRlZmF1bHQifQ.eyJpc3MiOiJodHRwczovL2F1dGguYW5pbWVvbnNlbi54eXovIiwiYXVkIjoiaHR0cHM6Ly9hcGkuYW5pbWVvbnNlbi54eXoiLCJpYXQiOjE2NzQwNDgzMDAsImV4cCI6MTY3NDY1MzEwMCwic3ViIjoiMDZkMjJiOTYtNjNlNy00NmE5LTgwZmMtZGM0NDFkNDFjMDM4LmNsaWVudCIsImF6cCI6IjA2ZDIyYjk2LTYzZTctNDZhOS04MGZjLWRjNDQxZDQxYzAzOCIsImd0eSI6ImNsaWVudF9jcmVkZW50aWFscyJ9.QjWtxXbWWQLrupXKwXNPR11fQddUauO-cXFMsxISBpcSXxbsFpwZTqmJrT8nbF9ZsxGPCGOX6AqzupGHY66SCP_vf01XpKi-8yxvb_jfcwW4-DA8IWh-bar1zpgyaVScCv1bh91OlLTulxAIkg0W_jfbEh6JYhMTZWBy1b7i-UONX4E-4vblhu3R9CGw2_pbF74IlPDDAPmHHsAF67O9Nx7TarQdvcUwCRzHFmyzyxa_3oZ4Hb_9LeUstINMWi0CM_jursyX4cw-t6XlPOdg41ii4VWHwk0zfQNzSiAPfhLn7tdFrLvYo1ap1MEx60dsS5kWVaJp36AJTjipObqKlQ";

// Since all network calls are cached, we don't care about caching the individual tokens.
pub async fn get_search_token(client: &ClientWithMiddleware) -> Option<String> {
    let res = client
        .get(format!("https://www.{}", host))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let html = Html::parse_document(res.as_str());
    let selector = Selector::parse("[name=\"ao-search-token\"]").unwrap();
    let token = html.select(&selector).next()?.value().attr("content");

    match token {
        Some(token) => Some(format!("Bearer {}", token)),
        None => None,
    }
}

pub async fn search(args: (&ClientWithMiddleware, &str)) -> Option<Vec<SearchResult>> {
    let (client, query) = args;
    let Some(token) = get_search_token(client).await else {
        crate::terminal::error("Failed to retrieve the search token");
        return None;
    };

    let mut json = HashMap::new();
    json.insert("q", query);

    let res = client
        .post(format!("https://search.{}/indexes/content/search", host))
        .header(AUTHORIZATION, token)
        .json(&json)
        .send().await.ok()? else {
            return None;
        };

    let json = res.json::<SearchReponse>().await.ok()? else {
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

    let Ok(ref id_regex) = regex::Regex::new(r#"animeonsen.xyz/watch/(.+?)\?episode=(\d+)"#) else {
        return None;
    };

    let Some(ref id) = id_regex.captures(url) else {
        return None;
    };

    let res = client
        .get(format!(
            "https://api.{}/v4/content/{}/video/{}",
            host,
            id.index(1),
            id.index(2),
        ))
        .header(AUTHORIZATION, api_authentication_header)
        .send()
        .await
        .ok()?;

    let json = res.json::<VideoResponse>().await.ok()?;

    Some(vec![StreamLink {
        url: json.clone().uri?.stream?,
        title: format!(
            "{} - {} - {}",
            json.clone().metadata?.content_title_en.unwrap_or(
                json.clone()
                    .metadata?
                    .content_title
                    .unwrap_or("N/A".to_string())
            ),
            id.index(2),
            json.clone()
                .metadata?
                .episode?
                .1
                .content_title_episode_en
                .unwrap_or(
                    json.clone()
                        .metadata?
                        .episode?
                        .1
                        .content_title_episode_jp
                        .unwrap_or("N/A".to_string())
                )
        )
        .to_string(),
        external_sub_url: json.clone().uri?.subtitles?.en_us?,
        is_direct: true,
    }])
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoResponse {
    pub metadata: Option<Metadata>,
    pub uri: Option<Uri>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub content_id: Option<String>,
    pub content_title: Option<String>,
    pub content_title_en: Option<String>,
    pub data_type: Option<String>,
    pub is_movie: Option<bool>,
    pub subtitle_support: Option<bool>,
    pub total_episodes: Option<i64>,
    pub next_season: Option<String>,
    pub mal_id: Option<i64>,
    pub episode: Option<(i32, EpisodeClass, HashMap<String, EpisodeClass>)>,
    pub subtitles: Option<Subtitles>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeClass {
    #[serde(rename = "contentTitle_episode_jp")]
    pub content_title_episode_jp: Option<String>,
    #[serde(rename = "contentTitle_episode_en")]
    pub content_title_episode_en: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subtitles {
    #[serde(rename = "en-US")]
    pub en_us: Option<String>,
    #[serde(rename = "es-LA")]
    pub es_la: Option<String>,
    #[serde(rename = "pt-BR")]
    pub pt_br: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Uri {
    pub stream: Option<String>,
    pub subtitles: Option<Subtitles>,
}
