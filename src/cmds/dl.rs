use clap::ArgMatches;

use crate::providers;

pub fn command(args: &ArgMatches) -> i16 {
    let query = args.value_of("query").unwrap();

    println!("Query: {}", query);
    return 0; // Ok
}
