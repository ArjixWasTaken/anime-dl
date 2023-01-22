use anyhow::{anyhow, bail, Result};
use clap::{ArgMatches, SubCommand};
use reqwest_middleware::ClientWithMiddleware;

pub async fn update(args: &ArgMatches<'_>) -> Result<()> {
    // TODO: Use the jaemk/self_update crate to implement this.
    bail!("Whine about it, lul.")
}

pub async fn test(args: &ArgMatches<'_>) -> Result<()> {
    bail!("Whine about it, lul.")
}

pub async fn command(client: &ClientWithMiddleware, args: &ArgMatches<'_>) -> Result<()> {
    match args.subcommand() {
        ("update", Some(sub_args)) => update(sub_args).await?,
        ("test", Some(sub_args)) => test(sub_args).await?,
        _ => (),
    }

    Ok(())
}
