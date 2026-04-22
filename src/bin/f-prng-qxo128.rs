use anyhow::{Context, Result};
use softrng::qxo128::{Q128Map, Qxo128, Q128};
use softrng::BUFLEN;
use std::io::{self, Read, Write};

fn main() -> Result<()> {
    let mut seed_bytes = vec![0u8; softrng::qxo128::MAP_SIZE];
    io::stdin().lock().read_exact(&mut seed_bytes)
        .context("Failed to read 4 MiB seed from stdin")?;

    let map = Q128Map::from_bytes(&seed_bytes)?;
    let qxo = Qxo128::new(map);
    let mut stdout = io::stdout().lock();
    let mut out_buf = vec![Q128::default(); BUFLEN];
    let mut idx: u128 = 0;

    loop {
        qxo.fill_chunks(&mut out_buf, idx);
        stdout.write_all(bytemuck::cast_slice(&out_buf))?;
        idx = idx.wrapping_add(BUFLEN as u128);
    }
}