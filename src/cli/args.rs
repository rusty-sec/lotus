use clap::{App, Arg, ArgMatches, Command};

pub fn cmd_args() -> ArgMatches {
    App::new("Lotus")
        .version("0.2-beta")
        .author("Khaled Nassar <knassar702@gmail.com>")
        .about("Fast Web Security Scanner written in Rust based on Lua Scripts ")
        .subcommands(vec![
            Command::new("urls")
                .about("working with urls only")
                .arg(
                    Arg::with_name("redirects")
                        .help("Set limit of http redirects")
                        .long("redirects")
                        .default_value("10")
                    )
                .arg(
                    Arg::with_name("timeout")
                        .help("Set Connection timeout")
                        .long("timeout")
                        .default_value("10")
                    )
                .arg(
                    Arg::with_name("proxy")
                        .help("Forward all connection through a proxy (eg: --proxy http://localhost:8080)")
                        .required(false)
                        .takes_value(true)
                        .validator(url::Url::parse)
                        .long("proxy")
                    )
                .arg(
                    Arg::with_name("workers")
                        .help("Number of works of urls")
                        .short('w')
                        .takes_value(true)
                        .default_value("10")
                        .long("workers"),
                )
                .arg(
                    Arg::with_name("log")
                        .help("Save all lots to custom file")
                        .takes_value(true)
                        .short('l')
                        .long("log"),
                )
                .arg(
                    Arg::with_name("script_threads")
                        .help("Workers for lua scripts")
                        .short('t')
                        .long("script-threads")
                        .takes_value(true)
                        .default_value("5"),
                )
                .arg(
                    Arg::with_name("scripts")
                        .help("Path of scripts dir")
                        .takes_value(true)
                        .short('s')
                        .long("scripts")
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .help("Path of the JSON output fiel")
                        .required(true)
                        .takes_value(true)
                        .long("output")
                        .short('o'),
                )
                .arg(Arg::with_name("nolog").help("no logging")),
        ])
        .get_matches()
}
