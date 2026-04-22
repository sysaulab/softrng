use anyhow::Result;
use softrng::seedy64::Seedy64;
use std::io::{self, Write};

/// Output an endless stream of 64-bit values, each generated from a fresh SEEDY64 instance.
/// This tests the distribution of the very first output after engine initialisation.
fn main() -> Result<()> {
    let mut stdout = io::stdout().lock();
    loop {
        // Create a new engine (nodes start at zero)
        let mut seedy = Seedy64::with_timings(10_000, 10_000);
        let mut buf = [0u8; 8];
        // Fill exactly 8 bytes (one u64) – this starts and stops the mixer threads
        seedy.fill_buffer(&mut buf);
        // Output the value
        stdout.write_all(&buf)?;
        stdout.flush()?;
        // Engine is dropped here, resetting completely
    }
}