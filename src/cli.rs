
use std::io::{self,Read,BufReader};
use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::{Regex,Captures,RegexBuilder};


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
            match opts.additional_args_needed() {
                0 => {
                    return Ok(WorkTodo::Work(Box::new(opts),regex,Vec::new()));
                },
                1 => {
                    let mut v = Vec::with_capacity(1);
                    if args.len() < 4 {
                        return Err(Cow::from(format!("required at least 3 args: '{} {} [FILE]' see '--help' for more info", &args[1], &args[2])));
                    } else {
                        v.push(args[3].to_string());
                    }
                    return Ok(WorkTodo::Work(Box::new(opts), regex, v));
                },
                2 => {
                    let mut v = Vec::with_capacity(2);
                    if args.len() < 4 {
                        return Err(Cow::from(format!("required at least 4 args: '{} {} [FILE] [FILE]' see '--help' for more info", &args[1], &args[2])));
                    } else {
                        v.push(args[3].to_string());
                    }
                    if args.len() < 5 {
                        return Err(Cow::from(format!("required at least 4 args: '{} {} {} [FILE]' see '--help' for more info", &args[1], &args[2], &args[3])));
                    } else {
                        v.push(args[4].to_string());
                    }
                    return Ok(WorkTodo::Work(Box::new(opts), regex, v));
                }
                _ => panic!()
            };
        } else {
            return Err(Cow::Borrowed("didn't understand that, see: '--help' for more info"));
        }
    }
}

pub enum WorkTodo {
    PrintHelp,
    PrintVersion,
    Work(Box<InitialFlagOptions>,Regex,Vec<String>),
}
/*
impl WorkTodo {
    fn do_work(&self) -> Result<Cow<'static,str>,Cow<'static,str>> {
        match self {
            &Self::PrintHelp => Ok("help text"),
            &Self::PrintVersion => Ok("version"),
            &Self::Work(ref cli, ref regex, ref opts) => {
                let reader = cli.open_input(opts).map_err(|e| Cow::Owned(format!("{:?}", e)))?;
                let mut s = String::with_capacity(4096);
            }
        }
    }
}
*/



#[allow(dead_code)]
#[derive(PartialEq,Eq,PartialOrd,Ord,Debug,Clone)]
pub struct InitialFlagOptions {
    input: Input,
    output: Output,
    matching: Matching,
    literal_match: bool,
    nice: bool,
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
                .dot_matches_new_line(if self.matching.is_multi_line() { if self.dot_matches_newline { false } else { true } } else { false })
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

    fn open_input(&self, opts: &[String]) -> Result<BufReader<Box<dyn Read>>,io::Error> {
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

