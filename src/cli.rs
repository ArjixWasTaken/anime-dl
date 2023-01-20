use clap::{App, Arg, SubCommand};

pub const PROVIDERS: &'static [&'static str] = &["animeonsen", "yugen"];

pub fn build_cli() -> App<'static, 'static> {
    App::new("anime")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommands(vec![
            SubCommand::with_name("dl")
                .about("Download anime")
                .arg(
                    Arg::with_name("query")
                        .help("the name of the anime")
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
                        .help("choice to start downloading given anime number")
                        .default_value("-1")
                        .use_delimiter(false)
                        .required(true),
                )
                .arg(
                    Arg::with_name("episode")
                        .short("e")
                        .help("the episode range to download")
                        .default_value("1:")
                        .use_delimiter(false)
                        .required(true),
                ),
            SubCommand::with_name("watch")
                .about("Watch anime")
                .arg(
                    Arg::with_name("query")
                        .help("the name of the anime")
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
                        .help("choice to start downloading given anime number")
                        .default_value("-1")
                        .use_delimiter(false)
                        .required(true),
                )
                .arg(
                    Arg::with_name("episode")
                        .short("e")
                        .help("the episode range to download")
                        .default_value("1:")
                        .use_delimiter(false)
                        .required(true),
                ),
            SubCommand::with_name("search"),
        ])
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
}
