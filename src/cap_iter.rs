use std::collections::VecDeque;
use std::io::{self,Write};
use regex::{CaptureMatches,Captures};

use crate::cap_groups::CapGroup;


pub struct CapIter<'a,'r> {
    buffer: &'a str,
    caps: CaptureMatches<'r,'a>,
    caps_done: bool,
    old_start: usize,
    nice: bool,
    queue: VecDeque<CapOut<'a>>
}
impl<'a,'r> CapIter<'a,'r> {
    pub fn new(buffer: &'a str, caps: CaptureMatches<'r,'a>, nice: bool) -> Self {
        CapIter {
            buffer, nice, caps,
            caps_done: false,
            old_start: 0,
            queue: VecDeque::with_capacity(2),
        }
    }
}
impl<'a,'r> Iterator for CapIter<'a,'r> {
    type Item = CapOut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.caps_done && self.queue.is_empty() {
                return None;
            }
            match self.queue.pop_front() {
                Option::None => {
                    self.advance_one_capture();
                }
                Option::Some(x) => {
                    return Some(x);
                }
            };
        }
    }
}

pub enum CapOut<'a> {
    CopyText(&'a str),
    Group(Captures<'a>),
}
impl<'a> CapOut<'a> {

    pub fn output<'b>(&self, arg: &[CapGroup<'b>], buffer: &mut String) {
        match self {
            &Self::CopyText(s) => buffer.push_str(s),
            &Self::Group(ref caps) => CapGroup::output(arg, caps, buffer),
        }
    }

    pub fn stream_output<'b,W: Write>(&self, arg: &[CapGroup<'b>], output: &mut W) -> io::Result<()> {
        match self {
            &Self::CopyText(s) => output.write_all(s.as_bytes()),
            &Self::Group(ref caps) => CapGroup::steam_output(arg, caps, output),
        }
    }
}

impl<'a,'r> CapIter<'a,'r> {
    fn advance_one_capture(&mut self) {
        if self.caps_done {
            return;
        }
        let cap = match self.caps.next() {
            Option::None => {
                self.caps_done = true;
                if (self.old_start+1) <= self.buffer.len() && self.nice {
                    self.queue.push_back(CapOut::CopyText(slice_str(self.buffer, self.old_start, self.buffer.len()-1)));
                }
                return;
            }
            Option::Some(cap) => cap
        };

        let (start,end) = match cap.get(0) {
            Option::None => unsafe { std::hint::unreachable_unchecked() },
            Option::Some(ref m) => (m.start(),m.end()),
        };
        if start != self.old_start && self.nice {
            self.queue.push_back(CapOut::CopyText(slice_str(self.buffer,self.old_start,start-1)));
        }
        self.old_start = end;
        self.queue.push_back(CapOut::Group(cap));
    }
}


fn slice_str<'a>(arg: &'a str, start: usize, end: usize) -> &'a str {
    unsafe {
        std::str::from_utf8_unchecked(&arg.as_bytes()[start..=end])
    }
}
