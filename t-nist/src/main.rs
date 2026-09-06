use anyhow::Result;
use nistrs::prelude::*;
use std::io::{self, BufReader, Read};

const BUFFER_SIZE: usize = 1024 * 64;   // 64 KB I/O chunk
const MAX_DATA_BYTES: usize = 256 * 1024 * 1024; // 256 MB – adjust as needed

/// Read from stdin, but stop after MAX_DATA_BYTES to avoid OOM.
/// Returns (data, truncated_flag).
fn read_stdin_limited() -> Result<(Vec<u8>, bool)> {
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, io::stdin());
    let mut data = Vec::with_capacity(MAX_DATA_BYTES.min(1024 * 1024)); // start with 1 MB
    let mut buf = [0u8; BUFFER_SIZE];
    let mut total_read = 0;
    let mut truncated = false;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let space_left = MAX_DATA_BYTES - total_read;
        if n > space_left {
            // We would exceed the limit; take only what fits.
            data.extend_from_slice(&buf[..space_left]);
            total_read += space_left;
            truncated = true;
            // Drain remaining input to avoid broken pipe? Not needed; we just stop reading.
            break;
        } else {
            data.extend_from_slice(&buf[..n]);
            total_read += n;
        }
        if total_read >= MAX_DATA_BYTES {
            truncated = true;
            break;
        }
    }

    if data.is_empty() {
        anyhow::bail!("No data provided on stdin");
    }
    Ok((data, truncated))
}

fn main() -> Result<()> {
    let (data, truncated) = read_stdin_limited()?;
    if truncated {
        eprintln!("Warning: input exceeded {} bytes; truncated.", MAX_DATA_BYTES);
    }

    let bits = BitsData::from_binary(data);

    println!("--- NIST Statistical Test Suite (nistrs) ---");

    // 1. Frequency
    let (pass, p) = frequency_test(&bits);
    println!("{:<38} p={:.12} pass={}", "Frequency", p, pass);

    // 2. Block Frequency
    match block_frequency_test(&bits, 128) {
        Ok((pass, p)) => println!("{:<38} p={:.12} pass={}", "Block Frequency (m=128)", p, pass),
        Err(e) => eprintln!("Error in Block Frequency: {}", e),
    }

    // 3. Runs
    let (pass, p) = runs_test(&bits);
    println!("{:<38} p={:.12} pass={}", "Runs", p, pass);

    // 4. Longest Run of Ones
    match longest_run_of_ones_test(&bits) {
        Ok((pass, p)) => println!("{:<38} p={:.12} pass={}", "Longest Run of Ones", p, pass),
        Err(e) => eprintln!("Error in Longest Run of Ones: {}", e),
    }

    // 5. Cumulative Sums (forward & backward)
    let results = cumulative_sums_test(&bits);
    for (i, (pass, p)) in results.iter().enumerate() {
        let label = if i == 0 {
            "Cumulative Sums (forward)"
        } else {
            "Cumulative Sums (backward)"
        };
        println!("{:<38} p={:.12} pass={}", label, p, pass);
    }

    Ok(())
}
