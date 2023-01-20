#![allow(dead_code)]

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
    pub external_sub_url: String,
    pub is_direct: bool,
}
