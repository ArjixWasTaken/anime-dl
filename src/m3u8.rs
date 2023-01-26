use anyhow::{anyhow, bail, Result};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest_middleware::ClientWithMiddleware;

#[rustfmt::skip]
lazy_static! {
    static ref ENCRYPTION_DETECTION_REGEX: Regex = Regex::new("#EXT-X-KEY:METHOD=([^,]+),").unwrap();
    static ref ENCRYPTION_URL_IV_REGEX: Regex = Regex::new("#EXT-X-KEY:METHOD=([^,]+),URI=\"([^\"]+)\"(?:,IV=(.*))?").unwrap();
    static ref QUALITY_REGEX: Regex = Regex::new(r#"#EXT-X-STREAM-INF:(?:(?:.*?(?:RESOLUTION=\d+x(\d+)).*?\s+(.*))|(?:.*?\s+(.*)))"#).unwrap();
    static ref TS_EXTENSION_REGEX: Regex = Regex::new(r#"(.*\.ts.*|.*\.jpg.*)"#).unwrap();
}

pub fn extract_streams<S: AsRef<str>>(
    client: &ClientWithMiddleware,
    master_playlist: S,
    headers: Option<Vec<(HeaderName, HeaderValue)>>,
) -> Result<()> {
    if let Some(ext) = crate::utils::get_absolute_extension(master_playlist) {
        if !["m3u8", "M3U8"].contains(&ext.as_str()) {
            bail!("The filetype '{}' is an unsupported stream.", ext);
        }
    } else {
        bail!("Couldn't find the filetype.");
    }

    dbg!(headers);

    Ok(())
}
