
use std::mem::replace;
use std::collections::VecDeque;
use std::io::{self,Read,BufReader,BufRead};

pub struct BufferedReader<R: Read> {
    buffer: Vec<u8>,
    stack: VecDeque<io::Result<(String,bool)>>,
    reader: BufReader<R>,
    eol: &'static [u8],
    end: bool,
}

impl<R: Read> Iterator for BufferedReader<R> {
    type Item=io::Result<(String,bool)>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.end && self.stack.is_empty() && self.buffer.is_empty() {
                // we are the EOF
                // no more lines/errors
                // no more buffered data
                //
                // iteration is over
                return None;
            }

            match self.stack.pop_front() {
                Option::None => {
                    self.read_new_lines();
                    continue;
                }
                Option::Some(x) => {
                    return Some(x);
                }
            };
        }
    }
}

impl<R: Read> BufferedReader<R> {

    pub fn new(buffer: BufReader<R>, eol: &'static [u8]) -> BufferedReader<R> {
        BufferedReader {
            buffer: Vec::new(),
            stack: VecDeque::new(),
            reader: buffer,
            eol: eol,
            end: false
        }
    }

    fn read_new_lines(&mut self) {
        loop {
            if self.end && self.stack.is_empty() && self.buffer.is_empty() {
                // nothing more to iterate
                // iterator is over
                return;
            }
            if !self.stack.is_empty() {
                // there are lines to return
                return;
            }
            self.read_until_approx_eol();    
            self.populate_lines_from_buffer();
        }
    }

    fn populate_lines_from_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let (splits,remainder) = split_iter_wrapper(self.buffer.as_slice(), self.eol);
        match splits {
            Option::None => { },
            Option::Some(v) => {
                for (line,eol_flag) in v {
                    let result = match std::str::from_utf8(line) {
                        Ok(s) => Ok((s.to_string(),eol_flag)),
                        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "stream does not contain valid utf8 data")),
                    };
                    self.stack.push_back(result);
                }
            }
        };
        match remainder {
            Option::None => {
                unsafe {
                    self.buffer.set_len(0);
                }
            }
            Option::Some(left_over) => {
                if left_over.len() == self.buffer.len() {
                    return;
                } else {
                    unsafe {
                        std::intrinsics::copy::<u8>(left_over.as_ptr(), self.buffer.as_ptr() as *mut u8, left_over.len());
                        self.buffer.set_len(left_over.len());
                    };
                }
            }
        };
        
    }

    fn read_until_approx_eol(&mut self) {
        if self.end {
            return;
        }

        if self.eol.is_empty() {
            unsafe { std::hint::unreachable_unchecked() };
        } if self.eol.len() == 1 {
            match self.reader.read_until(self.eol[0], &mut self.buffer) {
                Ok(x) => {
                    if x == 0 {
                        self.end = true;
                    }
                },
                Err(e) => {
                    self.stack.push_back(Err(e));
                },
            };
        } else {
            for b in self.eol.iter() {
                match self.reader.read_until(*b, &mut self.buffer) {
                    Ok(x) => {
                        if x == 0 {
                            self.end = true;
                        }
                    },
                    Err(e) => {
                        self.stack.push_back(Err(e));
                    }
                };
            }
        }
    }
}

fn split_iter_wrapper<'a, T: Eq+'static>(arg: &'a [T], needle: &'static [T]) -> (Option<Vec<(&'a [T], bool)>>,Option<&'a [T]>) {
    let mut items = SplitIterator::new(arg, needle)
        .collect::<Vec<_>>();
    let (last,terminated_by_eol) = items.pop().unwrap();
    if !terminated_by_eol {
        if items.is_empty() {
            (None,Some(last))
        } else {
            (Some(items),Some(last))
        }
    } else {
        items.push((last, terminated_by_eol));
        (Some(items),None)
    }
}

pub struct SplitIterator<'a,T: Eq +'static> {
    hayheap: &'a [T],
    needle: &'static [T],
}
impl<'a,T: Eq +'static> SplitIterator<'a,T> {
    pub fn new(hayheap: &'a [T],needle: &'static [T]) -> Self {
        Self { hayheap, needle }
    }
}
impl<'a,T: Eq +'static> Iterator for SplitIterator<'a,T> {
    type Item = (&'a [T],bool);
    fn next(&mut self) -> Option<Self::Item> {
        // sanity checks
        if self.needle.is_empty() || self.hayheap.is_empty() {
            return None;
        }

        // move haystack to the stack for easier manipluation
        let haystack = replace(&mut self.hayheap, &[]);
      
        if haystack.len() <= self.needle.len() {
            if haystack == self.needle {
                return Some((&[],true));
            } else {
                return Some((haystack,false));
            }
        }

        match haystack.windows(self.needle.len())
            .position(|window| window == self.needle)
        {
            Option::None => {
                return Some((haystack, false))
            }
            Option::Some(pos) => {
                let (before, after) = haystack.split_at(pos);
                let remainder = after.strip_prefix(self.needle).unwrap_or_else(|| after);
                self.hayheap = remainder;
                return Some((before,true));
            }
        };
    }
}

#[test]
fn test_split_iterator() {
    const SPLIT: &'static [u8] = &[0x0A];
    const X: &'static str = r#"
hello 
world
this
this
a
test

g
 "#;

    let iter = SplitIterator::new(X.as_bytes(), SPLIT).collect::<Vec<_>>();
    assert_eq!(iter.len(), 10);
    assert_eq!(iter[0].0.len(), 0);
    assert_eq!(iter[1].0.len(), 6);
    assert_eq!(iter[2].0.len(), 5);
    assert_eq!(iter[3].0.len(), 4);
    assert_eq!(iter[4].0.len(), 4);
    assert_eq!(iter[5].0.len(), 1);
    assert_eq!(iter[6].0.len(), 4);
    assert_eq!(iter[7].0.len(), 0);
    assert_eq!(iter[8].0.len(), 1);
    assert_eq!(&iter[8].0, b"g");
    assert_eq!(iter[9].0.len(), 1);
    assert_eq!(&iter[9].0, b" ");
    assert_eq!(iter[9].1, false);
}

#[test]
fn test_split_iterator_2() {
    const SPLIT: &'static [u8] = b"ee";
    const X: &'static str = "helloeeworldee";
    let iter = SplitIterator::new(X.as_bytes(), SPLIT).collect::<Vec<_>>();
    assert_eq!(iter.len(), 2);
    assert_eq!(iter[0].0, b"hello");
    assert_eq!(iter[0].1, true);
    assert_eq!(iter[1].0, b"world");
    assert_eq!(iter[1].1, true);
}
