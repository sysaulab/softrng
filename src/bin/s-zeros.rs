use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    let mut stdout = io::stdout().lock();
    let zero = [0u8;65536];

    loop {
        stdout.write_all(&zero)?;
    }
//    stdout.flush()?;
}