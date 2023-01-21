use crate::types::{StreamLink, SubtitleTrack};
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

pub async fn unpack(
    client: &ClientWithMiddleware,
    url: Url,
    stream: StreamLink,
) -> (Vec<StreamLink>, Vec<SubtitleTrack>) {
    (vec![], vec![])
}
