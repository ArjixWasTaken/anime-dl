use reqwest_middleware::ClientWithMiddleware;

use crate::types::StreamLink;

pub async fn unpack(client: &ClientWithMiddleware, stream: StreamLink) -> Vec<StreamLink> {
    // TODO: Implement this
    vec![]
}
