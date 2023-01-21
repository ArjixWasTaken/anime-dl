use reqwest_middleware::ClientWithMiddleware;

use crate::types::{StreamLink, SubtitleTrack};

pub async fn unpack(
    client: &ClientWithMiddleware,
    stream: StreamLink,
) -> (Vec<StreamLink>, Vec<SubtitleTrack>) {
    // TODO: Implement this
    (vec![], vec![])
}
