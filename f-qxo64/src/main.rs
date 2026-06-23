use anyhow::{Context, Result};
use qxo64::Qxo64;        // no more Q64Map / Q64 needed

pub const BUFLEN:usize = 1024 * 8;

use std::io::{self, Read, Write};

fn main() -> Result<()> {
    let mut seed_bytes = vec![0u8; qxo64::MAP_SIZE];
    io::stdin().lock().read_exact(&mut seed_bytes)
        .context("Failed to read 2 MiB seed from stdin")?;

    // Direct constructor from bytes – no separate map object.
    let qxo = Qxo64::new_from_data(&seed_bytes)?;

    let mut stdout = io::stdout().lock();
    // Buffer of u64 words instead of [u8; 8] arrays.
    let mut out_buf = vec![0u64; BUFLEN];
    let mut idx: u64 = 0;

    loop {
        // fill_u64s replaces fill_buffer
        qxo.fill_u64s(&mut out_buf, idx);
        // Cast the u64 slice to bytes – safe because Vec<u64> is properly aligned.
        stdout.write_all(bytemuck::cast_slice(&out_buf))?;
        idx = idx.wrapping_add(BUFLEN as u64);
    }
}