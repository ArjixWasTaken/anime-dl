use clap::ArgMatches;

use crate::providers;

pub fn command(args: &ArgMatches) -> i16 {
    let query = args.value_of("query").unwrap();

    println!("Query: {}", query);
    unsafe {
        println!(
            "Providers: {:?}",
            providers::get_providers()[0].command.call(()) // does not work for the love of god, we need another way of accessing the providers...
                                                           // Maybe a macro to generate a switch/match case for each provider, and we get the function by name or smth...
        );
    }

    return 0; // Ok
}
