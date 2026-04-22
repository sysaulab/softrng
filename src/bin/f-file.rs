use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, Read, Write};
use softrng::BUFFER_SIZE;

#[derive(Parser)]
#[command(name = "f-file", about = "Tee stdin to a file and stdout")]
struct Args {
    /// Output file path
    file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut file = File::create(&args.file)
        .with_context(|| format!("f-file: cannot create '{}'", args.file))?;

    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let n = stdin.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        stdout.write_all(&buffer[..n])?;
    }
    Ok(())
}
