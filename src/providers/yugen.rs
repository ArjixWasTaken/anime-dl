use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{header::HeaderValue, Client, Response};
use reqwest_middleware::ClientWithMiddleware;
use scraper::{element_ref::Select, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{AnimeEpisode, SearchResult, StreamLink, SubtitleTrack};

#[rustfmt::skip]
lazy_static! {
    static ref RE: Regex = Regex::new(r#"/watch/\d+/.*?/(\d+)/"#).unwrap();
    static ref NEXT_PAGE_RE: Regex = Regex::new(r#"page=(\d+)"#).unwrap();
    static ref EMBED_RE: Regex = Regex::new(r#"/e/(.*?)/"#).unwrap();
}

const host: &str = "yugen.to";
pub const test_episodes_link: &str = "https://yugen.to/anime/1428/overlord/";
pub const test_streams_link: &str = "https://yugen.to/watch/1428/overlord/1/";

pub async fn search(args: (&ClientWithMiddleware, &str)) -> Result<Vec<SearchResult>> {
    let (client, query) = args;
    let query = [("q", query)];
    let res = client
        .get(format!("https://{}/search/", host))
        .query(&query)
        .send()
        .await?
        .text()
        .await?;

    let html = Html::parse_document(res.as_str());
    let selector: Selector = Selector::parse("[class=\"anime-meta\"]").unwrap();

    Ok(html
        .select(&selector)
        .map(|element| SearchResult {
            title: element.value().attr("title").unwrap().trim().to_string(),
            url: format!("https://{}{}", host, element.value().attr("href").unwrap()),
            provider: "yugen".to_string(),
        })
        .collect::<Vec<SearchResult>>())
}

pub async fn get_episodes(args: (&ClientWithMiddleware, &str)) -> Result<Vec<AnimeEpisode>> {
    let (client, url) = args;
    let mut page = 1;

    let mut res: String = client
        .get(format!("{}watch?page={}", url, page))
        .send()
        .await?
        .text()
        .await?;

    let mut html = Html::parse_document(res.as_str());
    let mut episodes: Vec<AnimeEpisode> = Vec::new();

    let selector: Selector = Selector::parse("[class=\"ep-card\"] a:nth-child(2)").unwrap();
    let next_page_selector: Selector =
        Selector::parse("ul.pagination > li:last-child > a").unwrap();

    loop {
        for element in html.select(&selector) {
            let ep_num = RE
                .captures(element.value().attr("href").ok_or(anyhow!(""))?)
                .ok_or(anyhow!("No match found."))?
                .get(1)
                .unwrap()
                .as_str()
                .parse::<i32>()?;

            episodes.push(AnimeEpisode {
                title: element
                    .value()
                    .attr("title")
                    .unwrap()
                    .replacen(&format!("{} :", ep_num), "", 1)
                    .replace("None", "")
                    .trim()
                    .to_string(),
                url: format!("https://{}{}", host, element.value().attr("href").unwrap()),
                ep_num,
                provider: "yugen".to_string(),
            });
        }

        // A lot of breaks here, lol, should refactor them out...
        if let Some(next_page) = &html.select(&next_page_selector).next() {
            if let Some(href) = next_page.value().attr("href") {
                if let Some(next) = NEXT_PAGE_RE.captures(href) {
                    if let Ok(next) = next.get(1).unwrap().as_str().parse::<i32>() {
                        if page == next {
                            break;
                        }
                        page = next;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }

        res = client
            .get(format!("{}watch?page={}", url, page))
            .send()
            .await?
            .text()
            .await?;

        html = Html::parse_document(res.as_str());
    }

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

    let res: String = client.get(url).send().await?.text().await?;

    let html = Html::parse_document(res.as_str());
    let selector: Selector = Selector::parse("[id=\"main-embed\"]").unwrap();

    let id = EMBED_RE
        .captures(
            html.select(&selector)
                .next()
                .ok_or(anyhow!("Couldn't find the embed."))?
                .value()
                .attr("src")
                .unwrap(),
        )
        .ok_or(anyhow!("No match found"))?
        .get(1)
        .unwrap()
        .as_str();

    let res = client
        .post(format!("https://{}/api/embed/", host))
        .header("x-requested-with", "XMLHttpRequest")
        .form(&[("id", id), ("ac", "0")])
        .send()
        .await?;

    let json = res.json::<Embed>().await?;

    let mut streams = vec![];

    json.sources.map(|sources| {
        sources.iter().for_each(|source| {
            streams.push(StreamLink {
                title: source.name.clone().unwrap().to_string(),
                url: source.src.clone().unwrap().to_string(),
                is_direct: false,
                headers: None,
                quality: None,
            })
        });
    });

    json.hls.map(|hls| {
        hls.iter().for_each(|hls| {
            streams.push(StreamLink {
                title: "HLS".to_string(),
                url: hls.to_string(),
                is_direct: true,
                headers: None,
                quality: None,
            });
        });
    });

    Ok((streams, vec![]))
}

pub async fn test_streams(args: (&ClientWithMiddleware, &str)) -> Result<usize> {
    Ok(get_streams(args).await?.0.len())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Embed {
    pub message: Option<String>,
    pub thumbnail: Option<String>,
    pub multi: Option<Vec<Option<serde_json::Value>>>,
    pub sources: Option<Vec<Source>>,
    pub hls: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub name: Option<String>,
    pub src: Option<String>,
    #[serde(rename = "type")]
    pub source_type: Option<String>,
}
