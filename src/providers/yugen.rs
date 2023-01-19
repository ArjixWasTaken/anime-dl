use regex::Regex;
use reqwest::{Client, Response};
use reqwest_middleware::ClientWithMiddleware;
use scraper::{Html, Selector, element_ref::Select};

use crate::types::{AnimeEpisode, SearchResult, StreamLink};

const host: &str = "https://yugen.to";

pub async fn search(args: (&ClientWithMiddleware, &str)) -> Option<Vec<SearchResult>> {
    let (client, query) = args;
    let res: String = client
        .get(format!("{}/search/?q={}", host, query))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let html = Html::parse_document(res.as_str());
    let selector: Selector = Selector::parse("[class=\"anime-meta\"]").unwrap();
    if (html.select(&selector).count() == 0) {
        return None;
    };

    Some(
        html.select(&selector)
            .map(|element| SearchResult {
                title: element.value().attr("title").unwrap().to_string(),
                url: format!("{}{}", host, element.value().attr("href").unwrap()),
                provider: "yugen".to_string(),
            })
            .collect::<Vec<SearchResult>>(),
    )
}

pub async fn get_episodes(args: (&ClientWithMiddleware, &str)) -> Option<Vec<AnimeEpisode>> {
    let (client, url) = args;
    let res: String = client
        .get(format!("{}watch", url ))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let html = Html::parse_document(res.as_str());
    let selector: Selector = Selector::parse("[class=\"ep-card\"] a:nth-child(2)").unwrap();
    if (html.select(&selector).count() == 0) {
        return None;
    };
    
    let re = Regex::new(r#"/watch/\d+/.*?/(\d+)/"#).unwrap();

    let mut episodes: Vec<AnimeEpisode> = Vec::new();
    for element in html.select(&selector) {
        let ep_num = re.captures(element.value().attr("href")?)?.get(1)?.as_str().parse::<i32>().ok()?;
        episodes.push(AnimeEpisode {
            title: element.value().attr("title").unwrap().replacen(&format!("{} :", ep_num), "", 1).trim().to_string(),
            url: format!("{}{}", host, element.value().attr("href").unwrap()),
            ep_num: ep_num,
            provider: "yugen".to_string(),
        });
    }

    Some(episodes)

}

pub async fn get_streams(args: (&ClientWithMiddleware, &str)) -> Option<Vec<StreamLink>> {
    unreachable!()
}
