
use std::ffi::OsString;
use std::iter::IntoIterator;

use super::regex_syntax::{Parser};
use super::clap::{ArgMatches, Arg, App, SubCommand};


fn rust_regex_error_check(arg: String) -> Result<(), String> {
    Parser::new().parse(&arg).err().into_iter()
        .map(|parse_err| format!("{}", &parse_err))
        .next()
        .map_or_else(|| Ok(()), |err| Err(err))
}

pub fn parse<'a,S,T>(iter: T) -> ArgMatches<'a>
where
   S: Into<OsString> + Clone,
   T: IntoIterator<Item=S>
{
    App::new("sse")
        .author("cody <codylaeder@gmail.com>")
        .version("1.0.0")
        .about("simple stream editor")
        .set_term_width(80)
        .subcommand(SubCommand::with_name("input")
            .about("reads line-by-line from stdin")
            .help("reads line-by-line from stdin and prints ONLY transformed matches to stdout")
            .visible_aliases(&["i"])
            .display_order(1)
            .arg(Arg::with_name("capture-regex")
                .index(1)
                .takes_value(true)
                .required(true)
                .help("the regex to match on the input line")
                .validator(rust_regex_error_check))
            .arg(Arg::with_name("replacement-pattern")
                .index(2)
                .takes_value(true)
                .required(true)
                .help("$0 entire capture group, $1 first, $2 seconds, ... $$ = $")))
        .subcommand(SubCommand::with_name("input-ignore")
            .about("reads line-by-line from stdin")
            .help("reads line-by-line from stdin and prints ONLY transformed matches to stdout")
            .visible_aliases(&["ii", "input-ignored"])
            .display_order(2)
            .arg(Arg::with_name("capture-regex")
                .index(1)
                .takes_value(true)
                .required(true)
                .help("the regex to match on the input line")
                .validator(rust_regex_error_check))
            .arg(Arg::with_name("replacement-pattern")
                .index(2)
                .takes_value(true)
                .required(true)
                .help("$0 entire capture group, $1 first, $2 seconds, ... $$ = $")))
        .subcommand(SubCommand::with_name("gtmpl")
            .about("applies a go-template output")
            .help("reads input line-by-line and ouputs in a go template")
            .visible_aliases(&["g", "go-template"])
            .display_order(3)
            .arg(Arg::with_name("capture-regex")
                .index(1)
                .takes_value(true)
                .required(true)
                .help("the regex to match on the input line")
                .validator(rust_regex_error_check))
            .arg(Arg::with_name("go-template")
                .index(2)
                .takes_value(true)
                .required(true)
                .help("weirdness")))
 
        .get_matches_from(iter)
}
