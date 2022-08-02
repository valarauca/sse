# sse
Simple Stream Editor

sse is a stream editor that uses perl/python-esque regexes to lower the learning curve compared to sed. What I'm trying to say is I didn't want to learn sed's regex dialect.

### Example usage:

     $ sse -i [REGEX] [FORMAT STRING]
     $ sse -flf [FILE] [REGEX] [FORMAT STRING]

### CLI Options

(- can be omitted much like tar)

     -h       help
     -v       version
     -i       read STDIN passing matching string
     -in      read STDIN but pass non-matching lines (nice mode)
     -flo     read file line by line and write to stdout
     -flon    read line-by-line write to stdout pass non-matching line
     -flf     read file line by line and write back to it
     -flfn    read file line-by-line but return non-matching lines
     -flfw    read file line-by-line use Windows EOL
     -flfnw   read file line-by-line use nice mode use Windows EOL
     -fco     read file as a continous buffer 


### Regex Dialect:

Internally sse uses Rust Regexes (Thanks to Burnt Sushi, Alex Crichton, Huown, and other contributors). [Docs](https://doc.rust-lang.org/regex/regex/index.html) [Repo](https://github.com/rust-lang-nursery/regex)


### Format String Dialect:

- Single Digit Capture Groups: `%0` -> `%9`
- MultiDigit Capture Groups: `%<11>` -> `%<1009>`
- Labelled Capture Groups: `%<mygroup>`
- `%%` can be used to escape a capture group, solo `%` are not matched.


### Project License:


    Copyright (c) 2016-2022 William Cody Laeder

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
