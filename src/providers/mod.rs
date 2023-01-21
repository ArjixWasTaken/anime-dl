use crate::types::{AnimeEpisode, SearchResult, StreamLink, SubtitleTrack};
use reqwest::Client;
mod animeonsen;
mod yugen;
use anyhow::{anyhow, Context, Result};
use reqwest_middleware::ClientWithMiddleware;

macro_rules! provider_api {
    ($method:ident, $value:ident, $return:ty) => {
        pub async fn $method(
            client: &ClientWithMiddleware,
            provider_name: &str,
            $value: &str,
        ) -> Result<$return> {
            let call_expr_repr = format!(
                "{}::{}({}) [{}=\"{}\"]",
                provider_name,
                stringify!($method),
                stringify!(client, $value),
                stringify!($value),
                $value
            );

            crate::terminal::debug(format!("Attempting to execute '{}'", call_expr_repr));

            let result: Result<_> = (match provider_name {
                "animeonsen" => Ok(animeonsen::$method((client, &$value)).await),
                "yugen" => Ok(yugen::$method((client, &$value)).await),
                _ => Err(anyhow!("Unknown provider: {}", provider_name)),
            })?;

            crate::terminal::debug(format!("Successfully executed '{}'", call_expr_repr));

            result
        }
    };
}

provider_api!(search, query, Vec<SearchResult>);
#[rustfmt::skip]
provider_api!(get_episodes, anime_url, Vec<AnimeEpisode>);
#[rustfmt::skip]
provider_api!(get_streams, episode_url, (Vec<StreamLink>, Vec<SubtitleTrack>));
