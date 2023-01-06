use clap::ArgMatches;

use crate::providers;

pub fn command(args: &ArgMatches) -> i16 {
    let provider = args.value_of("provider").unwrap();
    let query = args.value_of("query").unwrap();

    providers::search(provider, query);
    return 0; // Ok
}
