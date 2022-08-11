extern crate regex;

pub mod cap_groups;
mod cli;
use cli::{WorkTodo,from_cli};
pub mod work;
pub mod buffered_reader;
pub mod cap_iter;

//declare messages
const VERS: &'static str = "1.0.0";
const MSG: &'static str ="
(not so) Simple Stream Editor

sse is a stream editor that uses perl/python-esque regexes
to lower the learning curve compared to sed.

CLI Options:

  Group 1: Input Mode (only 1)

    i: Input is stdin
    f: Input is a file

  Group 2: Optional Behavior modifiers (0-all)

    F: Literal Match
        Literal matching. No regex special characters
        will be used. Matches will be literal.

    n: Nice
        Program will emit non-matching sections/lines.

    S: Case insensitive
        Matching will not be concerned with case.
        Does not work with `F`.
    
    x: Ignore White Space

    G: Swap Greedy
        Normally `*` is greedy and `*?` is lazy
        With this flag that behavior will be the
        opposite.

    s: Dot Matches New Line
        This only works in continuous mode
        Dot will match a new line

    a: Ascii Only
        UTF-8 special characters that work with
        unicode character classes will have no
        effect.

  Group 3: Matching Mode (optional)

     c: continious
        The entire input will be considered when
        matching.

     l: line-by-line (default if ommited)
        Only individual lines will be matched on
        Defaults to 0xA as EOL character.

     lw: line-by-line (Windows EOL, 0xD 0xA)
     lm: line-by-line (Mac EOL, 0xD)
     lu: line-by-lien (Unix EOL, 0xA)
     li: line-by-line (IBM EOL, 0x15)
     lq: line-by-line (QNX EOL 0x1E)
     la: line-by-line (Acorn EOL 0xA 0xD)

  Group 4: Output Mode (optional)
     
     o: stdout (default if ommited)
     e: stderr
     f: file
     r: different file

Example usage:

$ sse -i [REGEX] [FORMAT STRING]
    `i`:     read from stdin
    default: read input line-by-line (with unix EOL)
    default: write to stdout

$ sse -in [REGEX] [FORMAT STRING]
    `i`:       read from stdin
    `n`:       non matching text will be copied without modification
    `default`: read input line-by-line (with unix EOL)
    `default`: write to stdout

$ sse -inlwf [REGEX] [FORMAT STRING] [FILE]
    `i`:       read from stdin
    `n`:       non matching text will be copied without modification
    `lw`:      line-by-line (with Windows EOL)
    `f`:       write to a file as output

$ sse -fFncf [REGEX] [FORMAT] [FILE]
    `f`:       read from file
    `F`:       `[REGEX]` & `[FORMAT]` will be applied literally
    `n`:       non matching text will be copied without modification
    `c`:       matching will be continuous line ending will be ignored.
    `f`:       write back to the same file

Regex Dialect:
Internally sse uses Rust Regexes (Thanks to Burnt Sushi, Alex Crichton, and Huown).

Format String Dialect:
- Single Digit Capture Groups: `%0` -> `%9`
- MultiDigit Capture Groups: `%<11>` -> `%<1009>`
- Labelled Capture Groups: `%<mygroup>`
- `%%` can be used to escape a capture group, solo `%` are not matched.

Example usage:
$ sse -i [REGEX] [FORMAT STRING]
$ sse -flf [FILE] [REGEX] [FORMAT STRING]";

//entry point
fn main() {
    match from_cli() {
        Ok(WorkTodo::PrintHelp) => {
            println!("{}", MSG);
        }
        Ok(WorkTodo::PrintVersion) => {
            println!("{}", VERS);
        }
        Ok(WorkTodo::Nothing) => { }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
}
