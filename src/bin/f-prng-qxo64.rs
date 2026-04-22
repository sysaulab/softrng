use anyhow::{Context, Result};
use softrng::qxo64::{Q64Map, Qxo64, Q64};
use softrng::BUFLEN;
use std::io::{self, Read, Write};

fn main() -> Result<()> {
    let mut seed_bytes = vec![0u8; softrng::qxo64::MAP_SIZE];
    io::stdin().lock().read_exact(&mut seed_bytes)
        .context("Failed to read 2 MiB seed from stdin")?;

    let map = Q64Map::from_bytes(&seed_bytes)?;
    let qxo = Qxo64::new(map);
    let mut stdout = io::stdout().lock();
    let mut out_buf = vec![Q64::default(); BUFLEN];
    let mut idx: u64 = 0;

    loop {
        qxo.fill_chunks(&mut out_buf, idx);
        stdout.write_all(bytemuck::cast_slice(&out_buf))?;
        idx = idx.wrapping_add(BUFLEN as u64);
    }
}