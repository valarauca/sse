extern crate regex;
use regex::{Regex};
use std::env::args;
use std::io::prelude::*;
use std::io;

mod cap_groups;
use cap_groups::{CapGroup};

//declare messages
const WRONG_ARGS: &'static str = "Was expecting an additional arg. Use sse -h for help.";
const HELP: &'static str = "Use sse -h for help.";
const VERS: &'static str = "0.0.2";
const MSG: &'static str ="
Simple Stream Editor

sse is a stream editor that uses perl/python-esque regexes
to lower the learning curve compared to sed.

CLI Options:
-h       help
-v       version
-i       read STDIN passing matching string
-in      read STDIN but pass non-matching lines
-flo     read file line by line and write to stdout
-flon    read line-by-line write to stdout pass non-matching line
-flf     read file line by line and write back to it
-flfn    read file line-by-line but return non-matching lines
-flfw    read file line-by-line use Windows EOL
-flfnw   read file line-by-line return non-matching use Windows EOL
-fco     read the file as a continous buffer


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


//So I don't have to write this a million times.
macro_rules! print_and_exit {
    ($arg: expr) => {
        println!("\n{}", $arg);
        ::std::process::exit(0i32);
    };

    ($arg: expr, $error: expr) => {
        println!("\n{}\n", $arg);
        println!("Error:  {:?}", $error );
        ::std::process::exit(0i32);
    };
}

/*
//Function that handles generating output
#[inline]
fn scan(buffer: &str,groups: &Captures) -> String {
    let mut c = buffer.chars();
    let mut out = String::with_capacity(4000);
    loop {
        match c.next() {
            Option::None => return out,
            Option::Some('%') => match c.next() {
                Option::None => { print_and_exit!("Error: % has to be followed by a char. See -h for help."); },
                Option::Some('0') => { push_capture!(0,out,groups); },
                Option::Some('1') => { push_capture!(1,out,groups); },
                Option::Some('2') => { push_capture!(2,out,groups); },
                Option::Some('3') => { push_capture!(3,out,groups); },
                Option::Some('4') => { push_capture!(4,out,groups); },
                Option::Some('5') => { push_capture!(5,out,groups); },
                Option::Some('6') => { push_capture!(6,out,groups); },
                Option::Some('7') => { push_capture!(7,out,groups); },
                Option::Some('8') => { push_capture!(8,out,groups); },
                Option::Some('9') => { push_capture!(9,out,groups); },
                Option::Some('%') => { out.push('%'); },
                Option::Some(x) => { print_and_exit!("Illegal Character.",x);}
            },
            Option::Some(x) => out.push(x)
        };
    }
}
*/

//Operations program can do
pub enum Ops {
    I(Regex,String,bool),
    FLO(String,Regex,String,bool),
    FLF(String,Regex,String,bool,bool),
    FLC(String,Regex,String)
}
impl Ops{

    //figure out what user wants to do
    fn new() -> Ops {
        let mut args: Vec<String> = args().skip(1).collect();
        args.reverse();
        if args.len() == 0 {
            print_and_exit!(HELP);
        }
        loop {
            match args.pop() {
                Option::None => {print_and_exit!(HELP);},
                Option::Some(ref x) if x == "--version" || x == "-v" => {print_and_exit!(VERS);},
                Option::Some(ref x) if x == "--help" || x == "-h" => {print_and_exit!(MSG);},
                Option::Some(ref x) if x == "-i" || x == "-in" || x == "i" || x == "in" => match args.pop() {
                        Option::Some(ref r) => match Regex::new(r) {
                            Ok(regex) => match args.pop() {
                                Option::Some(fmt) => match x.as_ref() {
                                    "-i" | "i" => return Ops::I(regex,fmt,false),
                                    "-in" | "in" => return Ops::I(regex,fmt,true),
                                    _ => unreachable!()
                                },
                                Option::None => {print_and_exit!(WRONG_ARGS);},
                            },
                            Err(e) => {print_and_exit!("Error occured building regex", e);},
                        },
                        Option::None => {print_and_exit!(WRONG_ARGS);},
                },
                Option::Some(ref x) if x =="-flo" || x=="-flon" || x=="flo" || x=="flon" => match args.pop(){
                    Option::Some(f) => match args.pop() {
                        Option::Some(ref r) => match Regex::new(r) {
                            Ok(regex) => match args.pop() {
                                Option::Some(fmt) => match x.as_ref() {
                                    "-flo" | "flo" => return Ops::FLO(f,regex,fmt,false),
                                    "-flon" | "flon" => return Ops::FLO(f,regex,fmt,true),
                                    _ => unreachable!()
                                },
                                Option::None => {print_and_exit!(WRONG_ARGS);},
                            },
                            Err(e) => {print_and_exit!("Error occured building regex", e);},
                        },
                        Option::None => {print_and_exit!(WRONG_ARGS);},
                    },
                    Option::None => {print_and_exit!(WRONG_ARGS);},
                },
                Option::Some(ref x) if
                        x == "-flf" ||x == "-flfc" || x== "-flfw" || x == "-flfcw" ||
                        x == "flf" || x == "flfc" || x == "flfw" || x == "flfcw" => match args.pop(){
                    Option::Some(f) => match args.pop() {
                        Option::Some(ref r) => match Regex::new(r) {
                            Ok(regex) => match args.pop() {
                                Option::Some(fmt) => match x.as_ref() {
                                    "-flf" | "flf" => return Ops::FLF(f,regex,fmt,false,false),
                                    "-flfc" | "flfc" => return Ops::FLF(f,regex,fmt,true,false),
                                    "-flfw" | "flfw"=> return Ops::FLF(f,regex,fmt,false,true),
                                    "-flfcw" | "flfcw" => return Ops::FLF(f,regex,fmt,true,true),
                                    _ => unreachable!(),
                                },
                                Option::None => {print_and_exit!(WRONG_ARGS);},
                            },
                            Err(e) => {print_and_exit!("Error occured building regex", e);},
                        },
                        Option::None => {print_and_exit!(WRONG_ARGS);},
                    },
                    Option::None => {print_and_exit!(WRONG_ARGS);},
                },
                Option::Some(ref x) if x == "-flo" || x == "flo" => match args.pop() {
                    Option::Some(f) => match args.pop() {
                        Option::Some(ref r) => match Regex::new(r) {
                            Ok(regex) => match args.pop() {
                                Option::Some(fmt) => return Ops::FLC(f,regex,fmt),
                                Option::None => { print_and_exit!(WRONG_ARGS);}
                            },
                            Err(e) => {print_and_exit!("Error occured building regex", e);}
                        },
                        Option::None => { print_and_exit!(WRONG_ARGS);}
                    },
                    Option::None => { print_and_exit!(WRONG_ARGS);}
                },
                Option::Some(ref x) if x == "-fco" || x == "fco" => match args.pop() {
                    Option::Some(f) => match args.pop(){
                        Option::Some(ref r) => match Regex::new(r) {
                            Ok(regex) => match args.pop() {
                                Option::Some(fmt) => return Ops::FLC(f,regex,fmt),
                                Option::None => {print_and_exit!(WRONG_ARGS);}
                            },
                            Err(e) => {print_and_exit!("Error occured building regex", e);}
                        },
                    Option::None => { print_and_exit!(WRONG_ARGS); }
                    
                    },
                    Option::None => { print_and_exit!(WRONG_ARGS); }
                },
                _ => {print_and_exit!("I don't understand that argument.");}
            }
        }
    }

    //do what user want
    fn exec(&self) {
        match self {
            &Ops::I(ref r,ref s,nice) => {
                let caps = CapGroup::build_groups(s);
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    match line {
                        Ok(ref x) => match r.captures(&x) {
                            Option::Some(ref c) => {
                                let mut s = String::with_capacity(4096);
                                CapGroup::output(caps.as_slice(), c, &mut s);
                                println!("{}", s);
                            },
                            Option::None => {
                                if nice {
                                    println!("{}", &x );
                                }
                            }
                        },
                        Err(e) => { print_and_exit!("Failed to open", e); }
                    }
                }
            },
            &Ops::FLO(ref f,ref r, ref s, nice) => {
                let caps = CapGroup::build_groups(s);
                let buff = match std::fs::read_to_string(f) {
                    Ok(x) => x,
                    Err(e) => { print_and_exit!("Failed to open", e); }
                };
                for line in buff.lines() {
                    match r.captures(&line) {
                        Option::Some(ref c) => {
                            let mut s = String::with_capacity(4096);
                            CapGroup::output(caps.as_slice(), c,&mut s);
                            println!("{}", s);
                        },
                        Option::None => {
                            if nice {
                                println!("{}", &line );
                            }
                        }
                    };
                }
            },
            &Ops::FLF(ref f, ref r, ref s, nice, win) => {
                let caps = CapGroup::build_groups(s);
                let output = match std::fs::read_to_string(f) {
                    Ok(x) => x,
                    Err(e) => { print_and_exit!("Failed to open", e); }
                };
                let mut buff_out = String::with_capacity(4096);
                for line in output.lines() {
                    match r.captures(&line) {
                        Option::Some(ref c) => {
                            CapGroup::output(&caps, c, &mut buff_out);
                            if win {
                                buff_out.push_str("\r\n");
                            } else {
                                buff_out.push('\n');
                            }
                        },
                        Option::None => {
                            if nice {
                                buff_out.push_str( &line );
                                if win {
                                    buff_out.push_str("\r\n");
                                } else {
                                    buff_out.push('\n');
                                }
                            }
                        }
                    };
                }
                match write_all(f, &buff_out) {
                    Ok(_) => { },
                    Err(e) => { print_and_exit!("Failed to write to file", e); }
                };
            },
            &Ops::FLC(ref f, ref r, ref s) => {
                let arg = match std::fs::read_to_string(f) {
                    Ok(buf) => buf,
                    Err(e) => { print_and_exit!("Failed to read file", e);}
                };
                let caps = CapGroup::build_groups(s);
                for cap in r.captures_iter(&arg) {
                    let mut s = String::with_capacity(4096);
                    CapGroup::output(&caps, &cap, &mut s); 
                    println!("{}", s);
                }
            }
        }
    }
}

fn write_all(path: &str, data: &String) -> Result<(),std::io::Error> {
    if data.len() == 0 {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new().truncate(true).create(true).write(true).open(path)?;
    f.write_all(data.as_bytes())?;
    f.sync_data()?;
    f.sync_all()?;
    Ok(())
}

//entry point
fn main() {
    let op = Ops::new();
    op.exec();
}
