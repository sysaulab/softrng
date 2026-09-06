use anyhow::{bail, Result};
use clap::Parser;
use seedy64::Seedy64;
use std::io::{BufWriter, Write};

pub const BUFFER_SIZE: usize = 64 * 1024;

/// Command line arguments
#[derive(Parser)]
struct Opts {
    /// Quality level: 0 (pass‑through), 1 (32‑bit mix), 2 (16‑bit mix), 3 (8‑bit mix)
    #[arg(long, default_value_t = 0)]
    quality: u8,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    // Validate quality
    if opts.quality > 3 {
        bail!("quality must be 0, 1, 2, or 3");
    }

    let mut seedy = Seedy64::new();
    let mut stdout = BufWriter::with_capacity(BUFFER_SIZE, std::io::stdout().lock());
    let mut raw_buf = vec![0u8; BUFFER_SIZE];

    loop {
        // Fill raw buffer with random bytes from the PRNG
		seedy.fill_buffer(&mut raw_buf);
		stdout.write_all(&raw_buf)?;
		stdout.flush()?;
    }
}
