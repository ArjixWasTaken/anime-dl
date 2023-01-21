#![allow(dead_code)]

use reqwest::header::{HeaderName, HeaderValue};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct AnimeEpisode {
    pub title: String,
    pub url: String,
    pub ep_num: i32,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct StreamLink {
    pub title: String,
    pub url: String,
    pub is_direct: bool,
    pub headers: Option<Vec<(HeaderName, HeaderValue)>>,
}

#[derive(Debug, Clone)]
pub enum SubtitleSource {
    Url(String),
    File(String),
}

#[derive(Debug, Clone)]
pub struct SubtitleTrack {
    pub lang: Option<String>,
    pub src: SubtitleSource,
    pub headers: Option<Vec<(HeaderName, HeaderValue)>>,
}
