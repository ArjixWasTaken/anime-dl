use reqwest_middleware::ClientWithMiddleware;
use scraper::{Html, Selector};
use url::Url;

use crate::types::{StreamLink, SubtitleTrack};

pub async fn unpack(
    client: &ClientWithMiddleware,
    url: Url,
    stream: StreamLink,
) -> (Vec<StreamLink>, Vec<SubtitleTrack>) {
    (vec![], vec![])
}
