
use std::io::{self,Write};

/// Flush trait is a super trait of write
pub trait FlushTrait: Write {
    fn trait_flush(&mut self) -> io::Result<()>;
}

impl FlushTrait for File {
    fn trait_flush(&mut self) -> io::Result<()> {
        self.fl
    }
}
