use clap::ArgMatches;
use reqwest::blocking::Client;

use crate::providers;
use crate::utils::search_results_to_table;

pub fn command(client: &Client, args: &ArgMatches) -> i16 {
    let provider = args.value_of("provider").unwrap();
    let query = args.value_of("query").unwrap();

    let Ok(search_results) = providers::search(client, provider, query) else {
        return 1; // Error
    };

    println!("{}", search_results_to_table(&search_results).to_string());
    let mut input = "".to_string();

    loop {
        if input.trim().len() >= 1 && input.trim().parse::<usize>().is_ok() {
            let idx = input.trim().parse::<usize>().unwrap();
            if idx > 0 && idx <= search_results.len() {
                break;
            }

            println!(
                "Please input a number within the range of 1-{}",
                search_results.len()
            );
        }

        input = casual::prompt("Select an anime [1]: ")
            .default("1".to_string())
            .get();
    }

    let chosen = search_results
        .get(input.trim().parse::<usize>().unwrap() - 1)
        .unwrap();

    println!("You selected {:#?}\n", chosen);

    providers::get_episodes(client, provider, chosen.url.as_str());

    return 0; // Ok
}
