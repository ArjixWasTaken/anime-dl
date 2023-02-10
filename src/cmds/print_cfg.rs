use anyhow::Result;
use clap::ArgMatches;
use reqwest_middleware::ClientWithMiddleware;

pub async fn command(
    config: &crate::config::Config,
    client: &ClientWithMiddleware,
    args: &ArgMatches<'_>,
) -> Result<()> {
    Ok(println!("{}", serde_yaml::to_string(config).unwrap()))
}
