use clap::{App, Arg, SubCommand};

pub const PROVIDERS: &'static [&'static str] = &["animeonsen", "yugen"];

pub fn build_cli() -> App<'static, 'static> {
    App::new("anime")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .subcommands(vec![
            SubCommand::with_name("dl")
                .about("Download anime")
                .arg(
                    Arg::with_name("query")
                        .help("The anime to search for.")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("provider")
                        .short("p")
                        .help("the anime provider (website) for search.")
                        .default_value(PROVIDERS.first().unwrap())
                        .possible_values(PROVIDERS),
                )
                .arg(
                    Arg::with_name("choice")
                        .short("c")
                        .help("If set, will skip the search select prompt and will choose the search result with this number.")
                        .default_value("-1")
                        .use_delimiter(false)
                        .required(true)
                        .hide_default_value(true),
                )
                .arg(
                    Arg::with_name("episodes")
                        .short("e")
                        .help("The episode range to download.")
                        .default_value("1:")
                        .use_delimiter(false)
                        .required(true),
                )
                .arg(
                    Arg::with_name("last-episode")
                        .conflicts_with("episodes")
                        .short("l")
                        .long("last-episode")
                        .takes_value(false)
                        .help("Automatically selects the last episode available."),
                ),
            SubCommand::with_name("watch")
                .about("Watch anime")
                .arg(
                    Arg::with_name("query")
                        .help("The anime to search for.")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("provider")
                        .short("p")
                        .help("The anime provider (website) for search.")
                        .default_value(PROVIDERS.first().unwrap())
                        .possible_values(PROVIDERS),
                )
                .arg(
                    Arg::with_name("choice")
                        .short("c")
                        .help("If set, will skip the search select prompt and will choose the search result with this number.")
                        .default_value("-1")
                        .use_delimiter(false)
                        .required(true)
                        .hide_default_value(true),
                )
                .arg(
                    Arg::with_name("episodes")
                        .short("e")
                        .help("The episode range to stream.")
                        .default_value("1:")
                        .use_delimiter(false)
                        .required(true),
                )
                .arg(
                    Arg::with_name("last-episode")
                        .conflicts_with("episodes")
                        .short("l")
                        .long("last-episode")
                        .takes_value(false)
                        .help("Automatically selects the last episode available."),
                ),
            SubCommand::with_name("self")
                .about("Miscellaneous commands")
                .alias("self_")
                .setting(clap::AppSettings::ArgRequiredElseHelp)
                .subcommands(vec![
                    SubCommand::with_name("update").about("Updates to the latest version."),
                    SubCommand::with_name("test")
                        .about("Tests all the providers and prints out which are working or not.")
                        .arg(
                            Arg::with_name("query")
                                .index(1)
                                .default_value("Overlord")
                                .help("The anime to search for."),
                        ),
                ]),
        ])
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
}
