use std::{collections::HashMap, ops::Index};

use anyhow::{anyhow, Result};
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::types::{AnimeEpisode, SearchResult, StreamLink, SubtitleSource, SubtitleTrack};

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
pub async fn get_search_token(client: &ClientWithMiddleware) -> Result<String> {
    let res = client
        .get(format!("https://www.{}", host))
        .send()
        .await?
        .text()
        .await?;
    let html = Html::parse_document(res.as_str());
    let selector = Selector::parse("[name=\"ao-search-token\"]").unwrap();
    let token = html
        .select(&selector)
        .next()
        .ok_or(anyhow!("Failed to find the search token."))?
        .value()
        .attr("content")
        .ok_or(anyhow!(
            "Found the search token, but the attr content does not exist..."
        ));

    Ok(format!("Bearer {}", token?))
}

pub async fn search(args: (&ClientWithMiddleware, &str)) -> Result<Vec<SearchResult>> {
    let (client, query) = args;
    let token = get_search_token(client).await?;

    let mut json = HashMap::new();
    json.insert("q", query);

    let res = client
        .post(format!("https://search.{}/indexes/content/search", host))
        .header(AUTHORIZATION, token)
        .json(&json)
        .send().await? else {
            return Err(anyhow!("Failed to make a connection."));
        };

    let json = res.json::<SearchReponse>().await?;

    if json.hits.is_none() {
        return Err(anyhow!("Unexpected API response."));
    }

    Ok(json
        .clone()
        .hits
        .ok_or(anyhow!("fuck"))?
        .iter()
        .map(|hit| SearchResult {
            title: hit.content_title.clone().unwrap().trim().to_string(),
            url: format!(
                "https://{}/details/{}",
                host,
                hit.content_id.clone().unwrap()
            ),
            provider: "animeonsen".to_string(),
        })
        .collect::<Vec<SearchResult>>())
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

pub async fn get_episodes(args: (&ClientWithMiddleware, &str)) -> Result<Vec<AnimeEpisode>> {
    let (client, url) = args;
    let id_regex = regex::Regex::new(r#"animeonsen.xyz/details/(.+)/?"#)?;

    let id = id_regex
        .captures(url)
        .ok_or(anyhow!("Couldn't find an id in the url."))?;

    let res = client
        .get(format!(
            "https://api.{}/v4/content/{}/episodes",
            host,
            id.index(1)
        ))
        .header(AUTHORIZATION, api_authentication_header)
        .send()
        .await?;

    let json = res.json::<EpisodesResponse>().await?;
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
    Ok(episodes)
}

pub async fn get_streams(
    args: (&ClientWithMiddleware, &str),
) -> Result<(Vec<StreamLink>, Vec<SubtitleTrack>)> {
    let (client, url) = args;

    let id_regex = regex::Regex::new(r#"animeonsen.xyz/watch/(.+?)\?episode=(\d+)"#)?;

    let id = id_regex
        .captures(url)
        .ok_or(anyhow!("Couldn't find an id in the url."))?;

    let res = client
        .get(format!(
            "https://api.{}/v4/content/{}/video/{}",
            host,
            id.index(1),
            id.index(2),
        ))
        .header(AUTHORIZATION, api_authentication_header)
        .send()
        .await?;

    let json = res.json::<VideoResponse>().await?;

    let stream = StreamLink {
        url: json.clone().uri.unwrap().stream.unwrap(),
        title: "AnimeOnsen".to_string(),
        is_direct: true,
        headers: None,
        quality: None,
    };

    let subs = json
        .clone()
        .uri
        .unwrap()
        .subtitles
        .map(|subtitles| {
            vec![
                ("en_us", subtitles.en_us),
                ("es_la", subtitles.es_la),
                ("pt_br", subtitles.pt_br),
            ]
            .iter()
            .filter(|x| x.1.is_some())
            .map(|x| SubtitleTrack {
                src: SubtitleSource::Url(x.1.clone().unwrap()),
                lang: Some(x.0.into()),
                headers: None,
            })
            .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok((vec![stream], subs))
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
