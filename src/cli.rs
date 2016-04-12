use clap::{App, Arg, ArgMatches};

pub fn args<'a>() -> ArgMatches<'a> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("dir")
             .index(1)
             .value_name("DIR")
             .help("Working directory [defaults to $PWD]"))
        .arg(Arg::with_name("editor")
             .short("e")
             .long("editor")
             .takes_value(true)
             .value_name("EDITOR")
             .help("Editor to use [defaults to $VISUAL, $EDITOR, or vi]"))
        .arg(Arg::with_name("all")
             .short("a")
             .long("all")
             .help("Don't ignore hidden files and directories"))
        .get_matches()
}
