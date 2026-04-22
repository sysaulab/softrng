use anyhow::{bail, Context, Result};
use clap::Parser;
use std::io::{self, Read, Write};
use softrng::BUFFER_SIZE;

#[derive(Parser)]
#[command(name = "f-limit", about = "Limit bytes from stdin")]
struct Args {
    /// Size with optional suffix (k/m/g/t or K/M/G/T)
    size: String,
}

fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.is_empty() {
        bail!("empty size");
    }

    // Find the suffix position
    let (num_part, suffix) = s.split_at(
        s.find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(s.len())
    );

    let base: f64 = num_part.parse()
        .with_context(|| format!("invalid number: '{}'", num_part))?;

    let multiplier = match suffix {
        "" | "b" | "B" => 1u64,
        "k" | "kb" => 1000,
        "m" | "mb" => 1_000_000,
        "g" | "gb" => 1_000_000_000,
        "t" | "tb" => 1_000_000_000_000,
        "p" | "pb" => 1_000_000_000_000_000,
        "e" | "eb" => 1_000_000_000_000_000_000,
        // Binary suffixes
        "K" | "KiB" => 1024,
        "M" | "MiB" => 1024 * 1024,
        "G" | "GiB" => 1024 * 1024 * 1024,
        "T" | "TiB" => 1024 * 1024 * 1024 * 1024,
        "P" | "PiB" => 1024 * 1024 * 1024 * 1024 * 1024,
        "E" | "EiB" => 1024 * 1024 * 1024 * 1024 * 1024 * 1024,
        _ => bail!("unknown suffix: '{}'", suffix),
    };

    Ok((base * multiplier as f64).round() as u64)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let max_bytes = parse_size(&args.size)?;

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut remaining = max_bytes;

    while remaining > 0 {
        let to_read = BUFFER_SIZE.min(remaining as usize);
        let n = stdin.read(&mut buffer[..to_read])?;
        if n == 0 {
            break;
        }
        stdout.write_all(&buffer[..n])?;
        remaining -= n as u64;
    }
    Ok(())
}
