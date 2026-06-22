use anyhow::Result;
use std::io::{self, copy, sink};

fn main() -> Result<()> {
    let mut stdin = io::stdin().lock();
    copy(&mut stdin, &mut sink())?;
    Ok(())
}