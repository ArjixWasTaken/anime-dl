#![allow(non_upper_case_globals)]

use crate::types::{AnimeEpisode, SearchResult, StreamLink};
use reqwest::Client;
mod animeonsen;
use reqwest_middleware::ClientWithMiddleware;

macro_rules! provider_api {
    ($method:ident, $value:ident, $return:ty) => {
        pub async fn $method(client: &ClientWithMiddleware, provider_name: &str, $value: &str) -> $return {
            let Some(ref result) = (match provider_name {
                "animeonsen" => Some(animeonsen::$method((client, &$value)).await?),
                _ => None,
            }) else {
                crate::terminal::error(format!("Provider '{}' not found", provider_name));
                return None;
            };

            crate::terminal::success(format!("Executed '{}::{}()'", provider_name, stringify!($method)));
            return Some(result.clone());
        }
    };
}

provider_api!(search, query, Option<Vec<SearchResult>>);
#[rustfmt::skip]
provider_api!(get_episodes, anime_url, Option<Vec<AnimeEpisode>>);
#[rustfmt::skip]
provider_api!(get_streams, episode_url, Option<Vec<StreamLink>>);
