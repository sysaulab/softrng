use anyhow::{Context, Result};
use pqxo64::PQXO64;
use qxo64::MAP_SIZE;
use std::io::Read;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <device_path> [num_threads]", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let num_threads = args.get(2)
        .map(|s| s.parse().context("Invalid thread count"))
        .transpose()?;

    let mut seed_bytes = vec![0u8; MAP_SIZE];
    std::io::stdin()
        .lock()
        .read_exact(&mut seed_bytes)
        .context("Failed to read 2 MiB seed from stdin")?;

    let pqxo = PQXO64::new(&seed_bytes, num_threads)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Write until the device is full (or interrupted) – use u64::MAX as "infinite".
    pqxo.fill_device(path, u64::MAX)
        .context("Failed to fill device")?;

    Ok(())
}