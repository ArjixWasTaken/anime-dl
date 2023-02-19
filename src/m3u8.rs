use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
use async_recursion::async_recursion;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest_middleware::ClientWithMiddleware;

#[rustfmt::skip]
lazy_static! {
    static ref ENCRYPTION_DETECTION_REGEX: Regex = Regex::new("#EXT-X-KEY:METHOD=([^,]+),").unwrap();
    static ref ENCRYPTION_URL_IV_REGEX: Regex = Regex::new("#EXT-X-KEY:METHOD=([^,]+),URI=\"([^\"]+)\"(?:,IV=(.*))?").unwrap();
    static ref QUALITY_REGEX: Regex = Regex::new(r#"#EXT-X-STREAM-INF:(?:(?:.*?(?:RESOLUTION=\d+x(\d+)).*?\s+(.*))|(?:.*?\s+(.*)))"#).unwrap();
    static ref TS_EXTENSION_REGEX: Regex = Regex::new(r#"(.*\.ts.*|.*\.jpg.*)"#).unwrap();
}

#[derive(Debug, Clone)]
pub struct M3u8Stream {
    pub url: String,
    pub quality: Option<String>,
    pub resolution: Option<String>,
    pub headers: Option<Vec<(HeaderName, HeaderValue)>>,
}

fn header_vec_to_map(headers: Vec<(HeaderName, HeaderValue)>) -> HeaderMap {
    headers
        .iter()
        .map(|(k, v)| (HeaderName::from(k), HeaderValue::from(v)))
        .collect()
}

fn get_parent_url(url: &str) -> String {
    let mut url = url
        .split("?")
        .take(1)
        .collect::<Vec<_>>()
        .get(0)
        .unwrap()
        .to_string();

    if url.ends_with('/') {
        url.pop();
    }

    url.split('/')
        .take(url.split('/').count() - 1)
        .collect::<Vec<_>>()
        .join("/")
}

fn is_partial_url(url: &str) -> bool {
    !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("://")
}

#[async_recursion]
pub async fn extract_streams(
    client: &ClientWithMiddleware,
    master_playlist: M3u8Stream,
    returnThis: Option<bool>,
) -> Result<Vec<M3u8Stream>> {
    if let Some(ext) = crate::utils::get_absolute_extension(&master_playlist.url) {
        if !["m3u8", "M3U8"].contains(&ext.as_str()) {
            bail!("The filetype '{}' is an unsupported stream.", ext);
        }
    } else {
        bail!("Couldn't find the filetype.");
    }

    let mut list = vec![];

    if returnThis.is_none() || (returnThis.is_some() && returnThis.unwrap()) {
        let mut clone = master_playlist.clone();
        if clone.quality.is_none() {
            clone.quality = Some("Auto".to_string());
        }
        list.push(clone);
    }

    let m3u8_parent = get_parent_url(&master_playlist.url);
    let res = client
        .get(&master_playlist.url)
        .headers(header_vec_to_map(
            master_playlist.headers.clone().unwrap_or_default(),
        ))
        .send()
        .await?
        .text()
        .await?;

    for cap in QUALITY_REGEX.captures_iter(&res) {
        let quality = cap.get(1).map(|x| x.as_str().to_string());
        let mut m3u8_link = cap.get(2).map(|x| x.as_str().to_string());
        if m3u8_link.is_none() {
            m3u8_link = cap.get(3).map(|x| x.as_str().to_string());
        }

        if let Some(mut m3u8_link) = m3u8_link {
            if crate::utils::get_absolute_extension(&m3u8_link) == Some("m3u8".to_string()) {
                if is_partial_url(&m3u8_link) {
                    m3u8_link = format!("{}/{}", m3u8_parent, m3u8_link);
                }

                list.extend(
                    extract_streams(
                        client,
                        M3u8Stream {
                            url: m3u8_link.clone(),
                            quality: quality.clone(),
                            resolution: None,
                            headers: master_playlist.headers.clone(),
                        },
                        Some(false),
                    )
                    .await
                    .unwrap_or_default(),
                );
            }

            let m3u8_stream = M3u8Stream {
                url: m3u8_link,
                quality,
                resolution: None,
                headers: master_playlist.headers.clone(),
            };

            list.push(m3u8_stream);
        }
    }

    Ok(list)
}
