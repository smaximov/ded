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
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Tell more details [disabled by default]"))
        .arg(Arg::with_name("yes")
             .short("y")
             .long("yes")
             .conflicts_with("no")
             .help("Assume answer `yes' to all questions [disabled by default]"))
        .arg(Arg::with_name("no")
             .short("n")
             .long("no")
             .conflicts_with("yes")
             .help("Assume answer `no' to all questions [disabled by default]"))
        .arg(Arg::with_name("dry-run")
             .long("dry-run")
             .help("Don't take any action, just show which files are modified"))
        .arg(Arg::with_name("match")
             .short("m")
             .long("match")
             .takes_value(true)
             .value_name("GLOB")
             .multiple(true)
             .number_of_values(1)
             .help("A glob to filter directory entries{n}\
                    Note: this options can occur multiple times")
             .next_line_help(true))
        .arg(Arg::with_name("tmp")
             .short("t")
             .long("tmp-path")
             .takes_value(true)
             .value_name("PATH")
             .help("A path to store ded's temp files [defaults to $TMPDIR/ded]"))
        .arg(Arg::with_name("only")
             .long("only")
             .takes_value(true)
             .value_name("dirs | files")
             .possible_values(&["dirs", "files"])
             .help("List only entries of the specified kind"))
        .get_matches()
}
