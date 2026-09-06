use anyhow::{bail, Result};
use clap::Parser;
use std::io::{self, Read, Write};

pub const BUFFER_SIZE: usize = 64 * 1024;

#[derive(Parser)]
#[command(name = "f-quality", about = "Reduce bit width of a binary stream")]
struct Args {
    /// Quality level: 0 (pass-through), 1 (32-bit), 2 (16-bit), 3 (8-bit)
    quality: u8,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.quality > 3 {
        bail!("quality must be 0, 1, 2, or 3");
    }

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut buf = vec![0u8; BUFFER_SIZE];
    // Align to 8 bytes for easier chunk processing
    let mut remainder = Vec::with_capacity(8);

    loop {
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            break;
        }

        // Combine with leftover bytes from previous read
        let mut data = std::mem::take(&mut remainder);
        data.extend_from_slice(&buf[..n]);

        // Process as many complete 64-bit words as possible
        let chunks = data.len() / 8;
        let mut pos = 0;
        for _ in 0..chunks {
            let word = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
            pos += 8;

            match args.quality {
                0 => {
                    stdout.write_all(&word.to_le_bytes())?;
                }
                1 => {
                    let mixed = ((word >> 32) ^ (word & 0xFFFF_FFFF)) as u32;
                    stdout.write_all(&mixed.to_le_bytes())?;
                }
                2 => {
                    let mixed32 = ((word >> 32) ^ (word & 0xFFFF_FFFF)) as u32;
                    let mixed16 = ((mixed32 >> 16) ^ (mixed32 & 0xFFFF)) as u16;
                    stdout.write_all(&mixed16.to_le_bytes())?;
                }
                3 => {
                    let mixed32 = ((word >> 32) ^ (word & 0xFFFF_FFFF)) as u32;
                    let mixed16 = ((mixed32 >> 16) ^ (mixed32 & 0xFFFF)) as u16;
                    let mixed8 = ((mixed16 >> 8) ^ (mixed16 & 0xFF)) as u8;
                    stdout.write_all(&[mixed8])?;
                }
                _ => unreachable!(),
            }
        }

        // Keep any leftover bytes (<8) for the next iteration
        if pos < data.len() {
            remainder.extend_from_slice(&data[pos..]);
        }
    }

    // If there are leftover bytes (<8) at EOF, we ignore them (not a full word)
    // Optionally you could error, but we just drop them.

    Ok(())
}
