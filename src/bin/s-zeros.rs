use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    let mut stdout = io::stdout().lock();
    let zero = 0u64;
    let bytes = zero.to_ne_bytes();

    loop {
        stdout.write_all(&bytes)?;
        stdout.flush()?;
    }
}