
use std::io::{stdin,BufRead,stdout,Write,ErrorKind};

use super::regex::{Regex};
use super::clap::{ArgMatches};
use super::gotmpl::{Value,Context};

pub fn cmd<'a>(args: &'a ArgMatches<'a>) -> Result<(),String> {
    // cli sanatizes this argument
    let regexp = Regex::new(args.value_of("capture-regex").unwrap()).unwrap();
    let format = args.value_of("replacement-pattern").unwrap();
    let mut line_buffer = String::with_capacity(4096);
    let stdin = stdin();
    let stdout = stdout();
    loop {
        match stdin.lock().read_line(&mut line_buffer) {
            Ok(_) => { },
            Err(ref e) => {
                match e.kind() {
                   ErrorKind::BrokenPipe |
                   ErrorKind::UnexpectedEof => return Ok(()),
                   _ => { }
                };
                return Err(format!("\nfailed to read from stdin\nos_code: {:?}\nerror: {:?}\n", e.raw_os_error(), e.kind()));
            }
        };
        let to_write = regexp.replace_all(&line_buffer,format);
        match stdout.lock().write_all(to_write.as_ref().as_bytes()) {
            Ok(_) => { },
            Err(ref e) => {
                return Err(format!("\nfailed to write to stdout\nos_code: {:?}\nerror: {:?}\n", e.raw_os_error(), e.kind()));
            }
        };
    }
}
