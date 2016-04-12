use clap::{App, Arg, ArgMatches};

pub fn args<'a>() -> ArgMatches<'a> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("dir")
             .index(1)
             .value_name("DIR")
             .help("working directory [defaults to $PWD]"))
        .get_matches()
}
