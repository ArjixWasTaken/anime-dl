use crate::types::StreamLink;
use async_recursion::async_recursion;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

mod goload;

#[async_recursion]
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

        let Ok(url) = Url::parse(&stream.url) else {
            continue;
        };

        let Some(hostname) = url.host_str() else {
            continue;
        };

        let hostname = hostname.to_string();

        unpacked_streams.extend(match hostname.as_str() {
            "goload.pro" => goload::unpack(client, stream).await,
            _ => vec![],
        });
    }

    if unpacked_streams.iter().all(|x| x.is_direct) {
        unpacked_streams
    } else {
        // An extractor gave more embed links, so we need to unpack them again
        unpack_streams(client, unpacked_streams).await
    }
}
