use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use softrng::BUFFER_SIZE;

#[derive(Parser)]
#[command(name = "s-file", about = "Output file contents from an optional offset")]
struct Args {
    /// Input file path
    file: String,
    /// Byte offset (default 0)
    offset: Option<u64>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let offset = args.offset.unwrap_or(0);

    let mut file = File::open(&args.file)
        .with_context(|| format!("s-file: cannot open '{}'", args.file))?;

    if offset > 0 {
        file.seek(SeekFrom::Start(offset))
            .with_context(|| format!("s-file: cannot seek to offset {}", offset))?;
    }

    let mut stdout = io::stdout().lock();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        stdout.write_all(&buffer[..n])?;
    }
    Ok(())
}
