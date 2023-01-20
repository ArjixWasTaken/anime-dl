use std::collections::HashMap;

use regex::Regex;
use reqwest::{header::HeaderValue, Client, Response};
use reqwest_middleware::ClientWithMiddleware;
use scraper::{element_ref::Select, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{AnimeEpisode, SearchResult, StreamLink};

const host: &str = "yugen.to";

pub async fn search(args: (&ClientWithMiddleware, &str)) -> Option<Vec<SearchResult>> {
    let (client, query) = args;
    let query = [("q", query)];
    let res = client
        .get(format!("https://{}/search/", host))
        .query(&query)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    
    let html = Html::parse_document(res.as_str());
    let selector: Selector = Selector::parse("[class=\"anime-meta\"]").unwrap();

    Some(
        html.select(&selector)
            .map(|element| SearchResult {
                title: element.value().attr("title").unwrap().to_string(),
                url: format!("https://{}{}", host, element.value().attr("href").unwrap()),
                provider: "yugen".to_string(),
            })
            .collect::<Vec<SearchResult>>(),
    )
}

pub async fn get_episodes(args: (&ClientWithMiddleware, &str)) -> Option<Vec<AnimeEpisode>> {
    let (client, url) = args;
    let res: String = client
        .get(format!("{}watch", url))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let html = Html::parse_document(res.as_str());
    let selector: Selector = Selector::parse("[class=\"ep-card\"] a:nth-child(2)").unwrap();

    let re = Regex::new(r#"/watch/\d+/.*?/(\d+)/"#).unwrap();
    let mut episodes: Vec<AnimeEpisode> = Vec::new();
    for element in html.select(&selector) {
        let ep_num = re
            .captures(element.value().attr("href")?)?
            .get(1)?
            .as_str()
            .parse::<i32>()
            .ok()?;
        episodes.push(AnimeEpisode {
            title: element
                .value()
                .attr("title")
                .unwrap()
                .replacen(&format!("{} :", ep_num), "", 1)
                .trim()
                .to_string(),
            url: format!("https://{}{}", host, element.value().attr("href").unwrap()),
            ep_num: ep_num,
            provider: "yugen".to_string(),
        });
    }

    Some(episodes)
}

pub async fn get_streams(args: (&ClientWithMiddleware, &str)) -> Option<Vec<StreamLink>> {
    let (client, url) = args;

    let res: String = client
        .get(format!("{}", url))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let html = Html::parse_document(res.as_str());
    let selector: Selector = Selector::parse("[id=\"main-embed\"]").unwrap();

    let re = Regex::new(r#"/e/(.*?)/"#).unwrap();
    let id = re
        .captures(html.select(&selector).next()?.value().attr("src")?)?
        .get(1)?
        .as_str();

    let res = client
        .post(format!("https://{}/api/embed/", host))
        .header("x-requested-with", "XMLHttpRequest")
        .form(&[("id", id), ("ac", "0")])
        .send()
        .await
        .ok()?;

    let json = res.json::<Embed>().await.ok()?;

    let mut streams = json
        .sources?
        .iter()
        .map(|source| StreamLink {
            title: source.name.clone().unwrap().to_string(),
            url: source.src.clone().unwrap().to_string(),
            external_sub_url: "".to_string(),
            is_direct: false,
        })
        .collect::<Vec<StreamLink>>();

    streams.extend(json.hls?.iter().map(|hls| StreamLink {
        title: "HLS".to_string(),
        url: hls.to_string(),
        external_sub_url: "".to_string(),
        is_direct: true,
    }));

    Some(streams)
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
