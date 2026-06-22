use anyhow::{anyhow, Result};
use getrandom::fill;
use std::io::{self, Write};

const CHUNK_SIZE: usize = 256;

fn main() -> Result<()> {
    let mut stdout = io::stdout().lock();
    let mut buf = [0u8; CHUNK_SIZE];

    loop {
        fill(&mut buf).map_err(|e| anyhow!("getrandom failed: {}", e))?;
        stdout.write_all(&buf)?;
        stdout.flush()?;
    }
}