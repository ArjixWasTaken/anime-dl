use crate::types::{StreamLink, SubtitleTrack};
use async_recursion::async_recursion;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

mod goload;

#[async_recursion]
pub async fn unpack_streams(
    client: &ClientWithMiddleware,
    streams: Vec<StreamLink>,
    subs: Vec<SubtitleTrack>,
) -> (Vec<StreamLink>, Vec<SubtitleTrack>) {
    let mut unpacked_streams = Vec::new();
    let mut unpacked_subs = subs;

    for stream in streams {
        if stream.is_direct {
            unpacked_streams.push(stream);
            continue;
        }

        let Ok(url) = Url::parse(&stream.url) else { continue; };
        let Some(hostname) = url.host_str() else { continue; };
        let hostname = hostname.to_string();

        let (streams_, subs_) = match hostname.as_str() {
            "goload.pro" => goload::unpack(client, stream).await,
            _ => (vec![], vec![]),
        };

        unpacked_streams.extend(streams_);
        unpacked_subs.extend(subs_);
    }

    if unpacked_streams.iter().all(|x| x.is_direct) {
        (unpacked_streams, unpacked_subs)
    } else {
        // An extractor gave more embed links, so we need to unpack them again
        unpack_streams(client, unpacked_streams, unpacked_subs).await
    }
}
