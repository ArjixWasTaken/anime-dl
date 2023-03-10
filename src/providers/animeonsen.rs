use std::{collections::HashMap, ops::Index};

use anyhow::{anyhow, bail, Result};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::types::{AnimeEpisode, SearchResult, StreamLink, SubtitleSource, SubtitleTrack};

#[rustfmt::skip]
lazy_static! {
    static ref ID_REGEX: Regex = Regex::new(r#"animeonsen.xyz/details/(.+)/?"#).unwrap();
    static ref EP_REGEX: Regex = Regex::new(r#"animeonsen.xyz/watch/(.+?)\?episode=(\d+)"#).unwrap();
}

pub const host: &str = "animeonsen.xyz";
pub const test_episodes_link: &str = "https://www.animeonsen.xyz/details/d5eRZVtbu86Kwy7E";
pub const test_streams_link: &str = "https://animeonsen.xyz/watch/d5eRZVtbu86Kwy7E?episode=1";

// Since all network calls are cached, we don't care about caching the individual tokens.
// Note: the above statement is invalid, for some fucking reason animeonsen isn't cached, that might be due to some headers.
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

pub async fn get_api_token(client: &ClientWithMiddleware) -> Result<String> {
    use base64::{engine::general_purpose, Engine as _};
    use cookie::Cookie;

    let res = client.get(format!("https://www.{}", host)).send().await?;

    let Some(cookie) = res.headers().get("set-cookie") else { bail!("AnimeOnsen: No cookies were found."); };
    let cookie = Cookie::parse(cookie.to_str()?)?.value().to_string();
    let cookie = urlencoding::decode(cookie.as_str())?.to_string();
    let mut decoded = general_purpose::STANDARD.decode(cookie).unwrap();
    decoded = decoded.into_iter().map(|x| x + 1).collect();

    let decoded = std::str::from_utf8(&decoded);

    Ok(format!("Bearer {}", decoded?))
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

    let id = ID_REGEX
        .captures(url)
        .ok_or(anyhow!("Couldn't find an id in the url."))?;
    let api_token = get_api_token(client).await?;

    let res = client
        .get(format!(
            "https://api.{}/v4/content/{}/episodes",
            host,
            id.index(1)
        ))
        .header(AUTHORIZATION, api_token)
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

pub async fn get_test_url(args: (&ClientWithMiddleware, &str)) -> Result<String> {
    match args.1 {
        "0" => Ok(test_episodes_link.to_string()),
        "1" => Ok(test_streams_link.to_string()),
        _ => bail!("Out of bounds."),
    }
}

pub async fn test_episodes(args: (&ClientWithMiddleware, &str)) -> Result<(i32, i32)> {
    let eps = get_episodes(args).await?;
    Ok((eps.len().try_into().unwrap(), 13))
}

pub async fn get_streams(
    args: (&ClientWithMiddleware, &str),
) -> Result<(Vec<StreamLink>, Vec<SubtitleTrack>)> {
    let (client, url) = args;

    let id = EP_REGEX
        .captures(url)
        .ok_or(anyhow!("Couldn't find an id in the url."))?;

    let api_token = get_api_token(client).await?;

    let res = client
        .get(format!(
            "https://api.{}/v4/content/{}/video/{}",
            host,
            id.index(1),
            id.index(2),
        ))
        .header(AUTHORIZATION, api_token)
        .send()
        .await?;

    let json = res.json::<VideoResponse>().await?;

    let stream = StreamLink {
        url: json.clone().uri.unwrap().stream.unwrap(),
        title: "AnimeOnsen".to_string(),
        is_direct: true,
        headers: Some(vec![(
            "Referer".parse()?,
            format!("https://www.{}", host).parse()?,
        )]),
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

pub async fn test_streams(args: (&ClientWithMiddleware, &str)) -> Result<usize> {
    Ok(get_streams(args).await?.0.len())
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
