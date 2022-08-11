
use regex::{Regex,Captures};
use lazy_static::lazy_static;

lazy_static! {
    static ref CAP_GROUP: Regex = Regex::new(r#"%(?P<escapegroup>%)?((<((?P<labelledgroup>[a-z][a-zA-Z0-9]+)|(?P<multidigit>[0-9]{2,}))>)|(?P<singledigit>[0-9]))"#).unwrap();
    static ref SPECIAL: Regex = Regex::new(r#"((?P<tab>\x5Ct)|(?P<newline>\x5Cn)|(?P<carriagereturn>\x5Cr)|(?P<verticaltab>\x5Cv)|(?P<backslash>\x5C\x5C)|(?P<unicode>\x5C(u|x|U)\{(?P<unicodevalue>[a-f0-9A-F]{1,8})\}))"#).unwrap();
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum CapGroup<'a> {
    MultiDigit(usize),
    SingleDigit(usize),
    Labelled(&'a str),
    Escape(&'a str),
    CopyFromInput(&'a str),
    SingleChar(char),
}
impl<'a> CapGroup<'a> {

    pub fn output(groups: &[CapGroup<'a>], caps: &Captures<'_>, buffer: &mut String) {
        for g in groups.iter() {
            match g {
                &CapGroup::MultiDigit(ref x) |
                &CapGroup::SingleDigit(ref x) => {
                    match caps.get(*x) {
                        Option::None => { }
                        Option::Some(ref m) => buffer.push_str(m.as_str()),
                    };
                }
                &CapGroup::Labelled(ref label) => {
                    match caps.name(label) {
                        Option::None => { },
                        Option::Some(ref m) => buffer.push_str(m.as_str()),
                    };
                }
                &CapGroup::Escape(x) |
                &CapGroup::CopyFromInput(x) => buffer.push_str(x),
                &CapGroup::SingleChar(c) => buffer.push(c),
            };
        }
    }

    pub fn build_groups(arg: &'a str) -> Vec<CapGroup<'a>> {
        let mut todo_list = Vec::with_capacity(1);
        let mut old_start = 0usize;
        for cap in CAP_GROUP.captures_iter(arg) {
            let (start,end) = match cap.get(0) {
                Option::None => unsafe { std::hint::unreachable_unchecked() },
                Option::Some(ref m) => (m.start(), m.end()),
            };
            if start != old_start {
                Self::build_spec_groups(slice_str(arg,old_start,start-1), &mut todo_list);
            }
            old_start = end;
            todo_list.push(CapGroup::build(cap));
        }
        if (old_start+1) <= arg.len() {
            Self::build_spec_groups(slice_str(arg, old_start, arg.len()-1), &mut todo_list);
        }
        todo_list
    }

    fn build(cap: Captures<'a>) -> CapGroup<'a> {
        let entire = match cap.get(0) {
            Option::None => unsafe { std::hint::unreachable_unchecked() },
            Option::Some(m) => m.as_str(),
        };
        if cap.name("escapegroup").is_some() {
            // remove first symbol in a fairly unsafe fashion.
            // we know the first character is an ASCII `%` so
            // we're just removing 1 byte and we know the match
            // is at least 3 bytes long, we this is safe, even
            // if rust doesn't think it is.
            //
            // going to do this as unsafely as possible as
            // a test to see when the MIRI jerks decides to
            // stop letting rust be cool.
            let x = unsafe {
                let rest = match entire.as_bytes().split_first() {
                    Option::Some((_, rest)) => rest,
                    Option::None => std::hint::unreachable_unchecked(),
                };
                std::str::from_utf8_unchecked(rest)
            };
            CapGroup::Escape(x)
        } else {
            match cap.name("labelledgroup")
                .map(|m| CapGroup::Labelled(m.as_str()))
                .into_iter()
                .chain( cap.name("multidigit")
                        .into_iter()
                        .filter_map(|m| usize::from_str_radix(m.as_str(), 10).ok())
                        .map(|d| CapGroup::MultiDigit(d)))
                .chain( cap.name("singledigit")
                        .into_iter()
                        .filter_map(|m| usize::from_str_radix(m.as_str(), 10).ok())
                        .map(|d| CapGroup::SingleDigit(d)))
                .next()
            {
                Option::None => unsafe { std::hint::unreachable_unchecked() },
                Option::Some(out) => out,
            }
        }
    }

    fn build_spec_groups(arg: &'a str, todo_list: &mut Vec<CapGroup<'a>>) {
        let mut old_start = 0usize;
        for cap in SPECIAL.captures_iter(arg) {
            let (start,end) = match cap.get(0) {
                Option::None => {
                    continue;
                },
                Option::Some(ref m) => (m.start(), m.end()),
            };
            if start != old_start {
                todo_list.push(CapGroup::CopyFromInput(slice_str(arg,old_start,start-1)));
            }
            old_start = end;
            todo_list.extend(CapGroup::build_spec(cap));
        }
        if (old_start+1) <= arg.len() {
            todo_list.push(CapGroup::CopyFromInput(slice_str(arg,old_start,arg.len()-1)));
        }
    }

    fn build_spec(cap: Captures<'a>) -> Option<CapGroup<'a>> {
        Option::None
            .into_iter()
            .chain(cap.name("tab").is_some()
                   .then(|| CapGroup::SingleChar('\t')))
            .chain(cap.name("newline").is_some()
                   .then(|| CapGroup::SingleChar('\n')))
            .chain(cap.name("carriagereturn").is_some()
                   .then(|| CapGroup::SingleChar('\r')))
            .chain(cap.name("verticaltab").is_some()
                   .then(|| CapGroup::SingleChar('\u{0B}')))
            .chain(cap.name("backslash").is_some()
                   .then(|| CapGroup::SingleChar('\u{0B}')))
            .chain(cap.name("unicodevalue")
                   .into_iter()
                   .filter_map(|m| u32::from_str_radix(m.as_str(),16).ok())
                   .filter_map(|u| char::from_u32(u))
                   .map(|c| CapGroup::SingleChar(c))
                   .next())
            .next()
    }
}

fn slice_str<'a>(arg: &'a str, start: usize, end: usize) -> &'a str {
    unsafe {
        std::str::from_utf8_unchecked(&arg.as_bytes()[start..=end])
    }
}


#[test]
fn test_cap_group_regex() {
    const NEEDS_TO_MATCH: &'static [&'static str] = &[
        "%1",
        "%<11>",
        "%<group1>",
        "%%<escape>",
    ];

    for item in NEEDS_TO_MATCH.iter() {
        assert!(CAP_GROUP.is_match(item));
    }
}
#[test]
fn spec_group_needs_to_match() {
    const NEEDS_TO_MATCH: &'static [&'static str] = &[
        r#"\t"#,
        r#"\n"#,
        r#"\r"#,
        r#"\v"#,
        r#"\\"#,
        r#"\u{2764}"#,
        r#"\x{2764}"#,
        r#"\U{2764}"#,
    ];
    for item in NEEDS_TO_MATCH {
        assert!(SPECIAL.is_match(item));
    }
}

#[test]
fn test_cap_group_construction() {
    let group = CapGroup::build(CAP_GROUP.captures("%1").unwrap());
    assert_eq!(group, CapGroup::SingleDigit(1));
    let group = CapGroup::build(CAP_GROUP.captures("%<201>").unwrap());
    assert_eq!(group, CapGroup::MultiDigit(201));
    let group = CapGroup::build(CAP_GROUP.captures("%<group6>").unwrap());
    assert_eq!(group, CapGroup::Labelled("group6"));
    let escaped = CapGroup::build(CAP_GROUP.captures("%%0").unwrap());
    assert_eq!(escaped, CapGroup::Escape("%0"));


    let group = CapGroup::build(CAP_GROUP.captures("hello world! %1").unwrap());
    assert_eq!(group, CapGroup::SingleDigit(1));
}


#[test]
fn build_cap_group() {
    let output = CapGroup::build_groups("hello %<11> world %2 weird%3%<world> pattern%4");
    assert_eq!(output[0], CapGroup::CopyFromInput("hello "));
    assert_eq!(output[1], CapGroup::MultiDigit(11));
    assert_eq!(output[2], CapGroup::CopyFromInput(" world "));
    assert_eq!(output[3], CapGroup::SingleDigit(2));
    assert_eq!(output[4], CapGroup::CopyFromInput(" weird"));
    assert_eq!(output[5], CapGroup::SingleDigit(3));
    assert_eq!(output[6], CapGroup::Labelled("world"));
    assert_eq!(output[7], CapGroup::CopyFromInput(" pattern"));
    assert_eq!(output[8], CapGroup::SingleDigit(4));
}

#[test]
fn test_last_char_breakage() {
    let output = CapGroup::build_groups(r#"hello %<11> weird%3%<world> pattern%4\tfoobar"#);
    assert_eq!(output[0], CapGroup::CopyFromInput("hello "));
    assert_eq!(output[1], CapGroup::MultiDigit(11));
    assert_eq!(output[2], CapGroup::CopyFromInput(" weird"));
    assert_eq!(output[3], CapGroup::SingleDigit(3));
    assert_eq!(output[4], CapGroup::Labelled("world"));
    assert_eq!(output[5], CapGroup::CopyFromInput(" pattern"));
    assert_eq!(output[6], CapGroup::SingleDigit(4));
    assert_eq!(output[7], CapGroup::SingleChar('\t'));
    assert_eq!(output[8], CapGroup::CopyFromInput("foobar"));
}

