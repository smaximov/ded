use std::io;
use std::io::{Write};

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
        .get_matches()
}

pub fn yes_or_no(prompt: &str, default: bool) -> io::Result<bool> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    let suggest = if default {
        "Y/n"
    } else {
        "y/N"
    };

    loop {
        try!(write!(stdout, "{} ({}) ", prompt, suggest));
        try!(stdout.flush());

        let mut input = String::new();

        try!(stdin.read_line(&mut input));

        let answer = input.trim().to_lowercase();

        match &answer[..] {
            "y" => return Ok(true),
            "n" => return Ok(false),
            "" => return Ok(default),
            _ => {
                try!(writeln!(stdout, "answer `y' or 'n'"));
            }
        }
    }
}
