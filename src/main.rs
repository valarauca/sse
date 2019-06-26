
use std::env;
use std::io::{Write,stdout,stderr};
use std::process::exit;

#[macro_use] extern crate lazy_static;
extern crate serde;
extern crate serde_yaml;
extern crate regex;
extern crate regex_syntax;
extern crate clap;
extern crate gtmpl;
extern crate gtmpl_value;

//#[macro_use] mod macros;
mod cli;
mod go_template;
mod input;
mod iinput;

fn main() {
   let args = cli::parse(env::args());
   let result = match args.subcommand() {
       ("input", Option::Some(ref args)) => input::cmd(args),
       ("iinput", Option::Some(ref args)) => iinput::cmd(args),
       (_,_) => {
          // this case will result in an error during
          // cli::parse so we dont need to handle it
          // here
          Ok(()) 
       },
   };
   let code = match result {
      Ok(()) => 0,
      Err(ref e) => {
          stderr().lock().write_all(e.as_bytes()).unwrap();
          1
      },
   };
   stderr().lock().flush().unwrap();
   stdout().lock().flush().unwrap();
   exit(code); 
}
