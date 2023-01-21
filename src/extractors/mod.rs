use crate::types::StreamLink;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;

pub async fn unpack_streams(
    client: &ClientWithMiddleware,
    streams: Vec<StreamLink>,
) -> Vec<StreamLink> {
    let mut unpacked_streams: Vec<StreamLink> = vec![];

    for stream in streams {
        if stream.is_direct {
            unpacked_streams.push(stream);
            continue;
        }

        // TODO: Detect and call the appropriate extractor.
    }

    unpacked_streams
}
