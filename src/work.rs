#[allow(unused_imports)]
use std::io::{self,Seek,Write,Read,BufWriter};

use regex::Regex;

use crate::{
    cli::{InitialFlagOptions},
    buffered_reader::{BufferedReader},
    cap_groups::CapGroup,
    cap_iter::CapIter,
};

pub fn do_work(
    opts: &InitialFlagOptions,
    regex: &Regex,
    caps: &[CapGroup<'_>],
    stack: &[String]
) -> io::Result<()> {
    let input_is_stdin = opts.input.is_stdin();
    let input = opts.input.open_input(stack)?;
    let input_streamable = opts.matching.build_input_stream(input);
    let output_streamable = opts.output.open_for_stream(input_is_stdin,stack)?;
    match (input_streamable,output_streamable) {
        (Ok((i,term)),Some(mut o)) => {
            do_streamable(i,&mut o, term, opts.nice, regex, caps)?;
            o.flush()?;
        }
        (Ok((i,term)),None) => {
            let mut cursor = std::io::Cursor::new(Vec::with_capacity(4096)); 
            do_streamable(i, &mut cursor, term, opts.nice, regex, caps)?;
            let mut output = opts.output.open_output(input_is_stdin, stack)?;
            let v = cursor.into_inner();
            output.write_all(v.as_slice())?;
            output.flush()?;
        },
        (Err(mut i),Some(mut o)) => {
            let mut s = String::with_capacity(4096);
            i.read_to_string(&mut s)?;
            let buffer = s.as_str();
            let cap_iter = regex.captures_iter(buffer);
            for item in CapIter::new(buffer, cap_iter,opts.nice) {
                item.stream_output(caps, &mut o)?;
            }
            o.flush()?;
        }
        (Err(mut i),None) => {
            let mut out_str = String::with_capacity(4096);

            let mut s = String::with_capacity(4096);
            {
                i.read_to_string(&mut s)?;
                std::mem::drop(i);
            }

            let buffer = s.as_str();
            let cap_iter = regex.captures_iter(buffer);
            for item in CapIter::new(buffer, cap_iter,opts.nice) {
                item.output(caps, &mut out_str);
            }

            let mut output = opts.output.open_output(input_is_stdin, stack)?;
            output.write_all(out_str.as_bytes())?;
            output.flush()?;
        }
    }
    Ok(())
}

pub trait MyTrait: Write {
    fn trait_flush(&mut self) -> io::Result<()>;
}
impl MyTrait for std::fs::File {
    fn trait_flush(&mut self) -> io::Result<()> {
        self.sync_data()?;
        self.sync_all()
    }
}
impl MyTrait for std::io::Stdout {
    fn trait_flush(&mut self) -> io::Result<()> {
        self.write_all(b"\n")?;
        self.flush()
    }
}
impl MyTrait for std::io::Stderr {
    fn trait_flush(&mut self) -> io::Result<()> {
        self.flush()
    }
}

fn do_streamable<R,W>(
    reader: BufferedReader<R>,
    writer: &mut W,
    term: &'static [u8],
    nice: bool, 
    regex: &Regex,
    caps: &[CapGroup<'_>],
) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    for res in reader {
        let (line,eol) = res?;
        match regex.captures(&line) {
            Option::None => {
                if nice {
                    writer.write_all(line.as_bytes())?;
                    if eol {
                        writer.write_all(term)?;
                    }
                }
            }
            Option::Some(ref c) => {
                CapGroup::steam_output(caps, c, writer)?;
                if eol {
                    writer.write_all(term)?;
                }
            }
        };
    }
    Ok(())
}
