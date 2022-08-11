
use std::io::{self,Read,BufReader,BufWriter};
use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::{Regex,Captures,RegexBuilder};

use crate::{
    cap_groups::{CapGroup},
    buffered_reader::BufferedReader,
    work::{MyTrait,do_work},
};


lazy_static! {
    static ref INITIAL_FLAG_MATCH: Regex = Regex::new(r#"^(?P<LeadingHypen>-)?((?P<stdin>i)|(?P<file>f))((?P<LiteralMatch>F)|(?P<nice>n)|(?P<CaseInSensitive>S)|(?P<IgnoreWhiteSpace>x)|(?P<SwapGreedy>G)|(?P<DotMatchesNewLine>s)|(?P<Ascii>a))*((?P<Continuous>c)|(?P<LineByLine>l((?P<WindowsEoL>w)|(?P<MacEoL>m)|(?P<UnixEoL>u)|(?P<IBM>i)|(?P<QNX>q)(?P<Acorn>a))?))?(?P<output>(?P<stdout>o)|(?P<stderr>e)|(?P<writeback>f)|(?P<redirect>r))?$"#).unwrap();
    static ref HELP: Regex = Regex::new(r#"^-?-[hH]([eE][lL][pP])?$"#).unwrap();
    static ref VERSION: Regex = Regex::new(r#"^-?-[vV](ersion)?$"#).unwrap();
}

pub fn from_cli() -> Result<WorkTodo,Cow<'static,str>> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() <= 1 {
        return Err(Cow::Borrowed("run --help for help"));
    } else {
        if HELP.is_match(&args[1]) {
            return Ok(WorkTodo::PrintHelp);
        }
        if VERSION.is_match(&args[1]) {
            return Ok(WorkTodo::PrintVersion);
        }
        if INITIAL_FLAG_MATCH.is_match(&args[1]) {
            let opts = InitialFlagOptions::new(&INITIAL_FLAG_MATCH.captures(&args[1]).unwrap());
            let regex = if args.len() < 3 {
                return Err(Cow::from(format!("required at least 2 args: '{} [REGEXP]' see '--help' for more info", &args[1])));
            } else {
                match opts.build_regex(&args[2]) {
                    Ok(x) => x,
                    Err(e) => return Err(Cow::from(e))
                }
            };
            let caps = if args.len() < 4 {
                return Err(Cow::from(format!("required at least 3 args: '{} {} [FORMAT STRING]' see '--help' for more info", &args[1], &args[2])));
            } else {
                if opts.literal_match {
                    vec![CapGroup::CopyFromInput(&args[3])]
                } else {
                    CapGroup::build_groups(&args[3])
                }
            };

            let optional_args = match opts.additional_args_needed() {
                1 => {
                    let mut v = Vec::with_capacity(1);
                    if args.len() < 5 {
                        return Err(Cow::from(format!("required at least 4 args: '{} {} {} [FILE]' see '--help' for more info", &args[1], &args[2], &args[3])));
                    } else {
                        v.push(args[4].to_string());
                    }
                    v
                },
                2 => {
                    let mut v = Vec::with_capacity(2);
                    if args.len() < 5 {
                        return Err(Cow::from(format!("required at least 5 args: '{} {} {} [FILE IN] [FILE OUT]' see '--help' for more info", &args[1], &args[2], &args[3])));
                    } else {
                        v.push(args[4].to_string());
                    }
                    if args.len() < 6 {
                        return Err(Cow::from(format!("required at least 4 args: '{} {} {} {} [FILE OUT]' see '--help' for more info", &args[1], &args[2], &args[3], &args[4])));
                    } else {
                        v.push(args[5].to_string());
                    }
                    v
                }
                _ => Vec::new(),
            };
            do_work(&opts, &regex, &caps, &optional_args)
                .map_err(|e| Cow::Owned(format!("{:?}", e)))?;
            return Ok(WorkTodo::Nothing);
        } else {
            return Err(Cow::Borrowed("didn't understand that, see: '--help' for more info"));
        }
    }
}

pub enum WorkTodo {
    PrintHelp,
    PrintVersion,
    Nothing,
}



#[allow(dead_code)]
#[derive(PartialEq,Eq,PartialOrd,Ord,Debug,Clone)]
pub struct InitialFlagOptions {
    pub input: Input,
    pub output: Output,
    pub matching: Matching,
    literal_match: bool,
    pub nice: bool,
    case_in_sensitive: bool,
    ignore_whitespace: bool,
    swap_greedy: bool,
    dot_matches_newline: bool,
    ascii_only: bool,
}
impl InitialFlagOptions {

    fn additional_args_needed(&self) -> usize {
        match (&self.input,&self.output) {
            (&Input::Stdin,&Output::SameFile) => 1,
            (&Input::Stdin,&Output::DifferentFile) => 1,
            (&Input::File,&Output::SameFile) => 1,
            (&Input::File,&Output::DifferentFile) => 2,
            _ => 0,
        }
    }

    fn build_regex(&self, arg: &str) -> Result<Regex,String> {
        if self.literal_match {
            Regex::new(&regex::escape(arg)).map_err(|e| format!("{:?}", e))
        } else {
            RegexBuilder::new(arg)
                .case_insensitive(self.case_in_sensitive)
                .multi_line(self.matching.is_multi_line())    
                .dot_matches_new_line(self.matching.is_multi_line() && self.dot_matches_newline)
                .swap_greed(self.swap_greedy)
                .ignore_whitespace(self.ignore_whitespace)
                .unicode(!self.ascii_only)
                .build()
                .map_err(|e| format!("{:?}", e))
        }
    }

    fn new(cap: &Captures<'_>) -> Self {
        Self {
            input: Input::new(cap),
            output: Output::new(cap),
            matching: Matching::new(cap),
            literal_match: cap.name("LiteralMatch").is_some(),
            nice: cap.name("nice").is_some(),
            case_in_sensitive: cap.name("CaseInSensitive").is_some(),
            ignore_whitespace: cap.name("IgnoreWhiteSpace").is_some(),
            swap_greedy: cap.name("SwapGreedy").is_some(),
            dot_matches_newline: cap.name("DotMatchesNewLine").is_some(),
            ascii_only: cap.name("ascii").is_some(),
        }
    }

    #[cfg(test)]
    const fn default() -> Self {
        Self {
            input: Input::Stdin,
            output: Output::Stdout,
            matching: Matching::LineByLine(Eol::Unix),
            literal_match: false,
            nice: false,
            case_in_sensitive: false,
            ignore_whitespace: false,
            swap_greedy: false,
            dot_matches_newline: false,
            ascii_only: false,
        }
    }

    #[cfg(test)]
    const fn set_input(mut self, input: Input) -> Self {
        self.input = input;
        self
    }
    #[cfg(test)]
    const fn set_output(mut self, output: Output) -> Self {
        self.output = output;
        self
    }
    #[cfg(test)]
    const fn set_matching(mut self, matching: Matching) -> Self {
        self.matching = matching;
        self
    }
    #[cfg(test)]
    const fn set_literal_match(mut self, literal_match: bool) -> Self {
        self.literal_match = literal_match;
        self
    }
    #[cfg(test)]
    const fn set_nice(mut self, nice: bool) -> Self {
        self.nice = nice;
        self
    }
    #[cfg(test)]
    const fn set_case_in_sensitive(mut self, case_in_sensitive: bool) -> Self {
        self.case_in_sensitive = case_in_sensitive;
        self
    }
    #[cfg(test)]
    const fn set_ignore_whitespace(mut self, ignore_whitespace: bool) -> Self {
        self.ignore_whitespace = ignore_whitespace;
        self
    }
    #[cfg(test)]
    const fn set_swap_greedy(mut self, swap_greedy: bool) -> Self {
        self.swap_greedy = swap_greedy;
        self
    }
    #[cfg(test)]
    const fn set_dot_matches_newline(mut self, dot_matches_newline: bool) -> Self {
        self.dot_matches_newline = dot_matches_newline;
        self
    }
    #[cfg(test)]
    const fn set_ascii_only(mut self, ascii_only: bool) -> Self {
        self.ascii_only = ascii_only;
        self
    }
}

#[test]
fn test_args() {
    const DUT: &'static [(&'static str, InitialFlagOptions)] = &[
        ("-i",InitialFlagOptions::default()),
        ("-in",InitialFlagOptions::default().set_nice(true)),
        ("-fnlo",InitialFlagOptions::default().set_input(Input::File).set_nice(true)),
        ("-fnlwo",InitialFlagOptions::default().set_nice(true).set_input(Input::File).set_matching(Matching::LineByLine(Eol::Windows))),
        ("-flf", InitialFlagOptions::default().set_input(Input::File).set_output(Output::SameFile)),
        ("-iFnSxGsacr",InitialFlagOptions::default()
            .set_literal_match(true)
            .set_nice(true)
            .set_case_in_sensitive(true)
            .set_ignore_whitespace(true)
            .set_swap_greedy(true)
            .set_dot_matches_newline(true)
            .set_ascii_only(true)
            .set_matching(Matching::Continuous)
            .set_output(Output::DifferentFile)),
    ];
    for (item,opts) in DUT {
        match INITIAL_FLAG_MATCH.captures(item) {
            Option::None => panic!("needs to match {:?}", item),
            Option::Some(ref built) => {
                let test_opts = InitialFlagOptions::new(built);
                if !opts.input.eq(&test_opts.input) {
                    panic!("{} -> input: {:?} != {:?}", item, &opts.input, &test_opts.input);
                }
                if !opts.output.eq(&test_opts.output) {
                    panic!("{} -> output: {:?} != {:?}", item, &opts.output, &test_opts.output);
                }
                if !opts.matching.eq(&test_opts.matching) {
                    panic!("{} -> matching: {:?} != {:?}", item, &opts.matching, &test_opts.matching);
                }
                if !opts.literal_match.eq(&test_opts.literal_match) {
                    panic!("{} -> literal_match: {:?} != {:?}", item, &opts.literal_match, &test_opts.literal_match);
                }
                if !opts.nice.eq(&test_opts.nice) {
                    panic!("{} -> nice: {:?} != {:?}", item, &opts.nice, &test_opts.nice);
                }
                if !opts.case_in_sensitive.eq(&test_opts.case_in_sensitive) {
                    panic!("{} -> case_in_sensitive: {:?} != {:?}", item, &opts.case_in_sensitive, &test_opts.case_in_sensitive);
                }
                if !opts.ignore_whitespace.eq(&test_opts.ignore_whitespace) {
                    panic!("{} -> ignore_whitespace: {:?} != {:?}", item, &opts.ignore_whitespace, &test_opts.ignore_whitespace);
                }
                if !opts.swap_greedy.eq(&test_opts.swap_greedy) {
                    panic!("{} -> swap_greedy: {:?} != {:?}", item, &opts.swap_greedy, &test_opts.swap_greedy);
                }
                if !opts.dot_matches_newline.eq(&test_opts.dot_matches_newline) {
                    panic!("{} -> dot_matches_newline: {:?} != {:?}", item, &opts.dot_matches_newline, &test_opts.dot_matches_newline);
                }
            }
        };
    }
}


#[allow(dead_code)]
#[derive(PartialEq,Eq,PartialOrd,Ord,Debug,Clone,Copy)]
pub enum Matching {
    Continuous,
    LineByLine(Eol),
}
impl Matching {

    pub fn build_input_stream<R: Read>(&self, input: BufReader<R>) -> Result<(BufferedReader<R>,&'static [u8]), BufReader<R>> {
        match self {
            &Self::Continuous => Err(input),
            &Self::LineByLine(ref eol) => {
                let term = eol.get_eol_bytes();
                Ok((BufferedReader::new(input, term), term))
            }
        }
    }

    fn is_multi_line(&self) -> bool {
        *self == Matching::Continuous
    }

    fn new(cap: &Captures<'_>) -> Self {
        cap.name("Continuous")
            .is_some()
            .then(|| Self::Continuous)
            .or_else(|| cap.name("LineByLine").is_some().then(|| Self::LineByLine(Eol::new(cap))))
            .unwrap_or_else(|| Self::LineByLine(Eol::Unix))
    }
}

#[allow(dead_code)]
#[derive(PartialEq,Eq,PartialOrd,Ord,Debug,Clone,Copy)]
pub enum Eol {
    Windows,
    Mac,
    Unix,
    Ibm,
    Qnx,
    Acorn,
}
impl Eol {

    fn get_eol_bytes(&self) -> &'static [u8] {
        const WINDOWS_EOL: &'static [u8] = &[ 0x0D, 0x0A];
        const MAC_EOL: &'static [u8] = &[0x0D];
        const UNIX_EOL: &'static [u8] = &[0x0A];
        const IBM_EOL: &'static [u8] = &[0x15];
        const QNX_EOL: &'static [u8] = &[0x1E];
        const ACORN_EOL: &'static [u8] = &[0x0A,0x0D];
        match self {
            &Self::Windows => WINDOWS_EOL,
            &Self::Mac => MAC_EOL,
            &Self::Unix => UNIX_EOL,
            &Self::Ibm => IBM_EOL,
            &Self::Qnx => QNX_EOL,
            &Self::Acorn => ACORN_EOL,
        }
    }

    fn new(cap: &Captures<'_>) -> Self {
        cap.name("WindowsEoL")
            .is_some()
            .then(|| Self::Windows)
            .or_else(|| cap.name("MacEoL").is_some().then(|| Self::Mac))
            .or_else(|| cap.name("UnixEoL").is_some().then(|| Self::Unix))
            .or_else(|| cap.name("IBM").is_some().then(|| Self::Ibm))
            .or_else(|| cap.name("QNX").is_some().then(|| Self::Qnx))
            .or_else(|| cap.name("Acorn").is_some().then(|| Self::Acorn))
            .unwrap_or_else(|| Self::Unix)
    }
}


#[allow(dead_code)]
#[derive(PartialEq,Eq,PartialOrd,Ord,Debug,Clone,Copy)]
pub enum Output {
    Stdout,
    Stderr,
    SameFile,
    DifferentFile
}
impl Output {

    pub fn open_output(
        &self,
        input_is_stdin: bool,
        args: &[String],
    ) -> Result<BufWriter<Box<dyn MyTrait>>,io::Error> {
        match self {
            &Self::Stdout => Ok(BufWriter::with_capacity(16 * 1024, Box::new(std::io::stdout()))),
            &Self::Stderr => Ok(BufWriter::with_capacity(16 * 1024, Box::new(std::io::stderr()))),
            &Self::SameFile |
            &Self::DifferentFile => {
                if input_is_stdin {
                    Ok(BufWriter::with_capacity(16 * 1024, Box::new(std::fs::OpenOptions::new().create(true).truncate(true).open(&args[0])?)))
                } else {
                    Ok(BufWriter::with_capacity(16 * 1024, Box::new(std::fs::OpenOptions::new().create(true).truncate(true).open(&args[1])?)))
                }
            },
        }
    }

    pub fn open_for_stream(
        &self,
        input_is_stdin: bool,
        args: &[String],
    ) -> Result<Option<BufWriter<Box<dyn MyTrait>>>,io::Error> {
        match self {
            &Self::Stdout => Ok(Some(BufWriter::with_capacity(16 * 1024, Box::new(std::io::stdout())))),
            &Self::Stderr => Ok(Some(BufWriter::with_capacity(16 * 1024, Box::new(std::io::stderr())))),
            &Self::SameFile => {
                if input_is_stdin {
                    Ok(Some(BufWriter::with_capacity(16 * 1024, Box::new(std::fs::OpenOptions::new().create(true).truncate(true).open(&args[0])?))))
                } else {
                    // writing back to same file
                    Ok(None)
                }
            },
            &Self::DifferentFile => {
                if input_is_stdin {
                    Ok(Some(BufWriter::with_capacity(16 * 1024, Box::new(std::fs::OpenOptions::new().create(true).truncate(true).open(&args[0])?))))
                } else {
                    Ok(Some(BufWriter::with_capacity(16 * 1024, Box::new(std::fs::OpenOptions::new().create(true).truncate(true).open(&args[1])?))))
                }
            },
        }
    }

    fn new(cap: &Captures<'_>) -> Self {
        cap.name("output")
            .is_some()
            .then(||
                cap.name("stdout").is_some().then(|| Output::Stdout)
                    .or_else(|| cap.name("stderr").is_some().then(|| Output::Stderr))
                    .or_else(|| cap.name("writeback").is_some().then(|| Output::SameFile))
                    .or_else(|| cap.name("redirect").is_some().then(|| Output::DifferentFile)))
            .flatten()
            .unwrap_or_else(|| Output::Stdout)
    }
}

#[allow(dead_code)]
#[derive(PartialEq,Eq,PartialOrd,Ord,Debug,Clone,Copy)]
pub enum Input {
    Stdin,
    File,
}
impl Input {

    pub fn is_stdin(&self) -> bool {
        *self == Self::Stdin
    }

    pub fn open_input(&self, opts: &[String]) -> Result<BufReader<Box<dyn Read>>,io::Error> {
        match self {
            &Self::Stdin => Ok(BufReader::with_capacity(32 * 1024, Box::new(std::io::stdin()))),
            &Self::File => Ok(BufReader::with_capacity(32 * 1024, Box::new(std::fs::File::open(&opts[0])?))),
        }
    }

    fn new(cap: &Captures<'_>) -> Self {
        if cap.name("stdin").is_some() {
            Self::Stdin
        } else {
            Self::File
        }
    }
}

