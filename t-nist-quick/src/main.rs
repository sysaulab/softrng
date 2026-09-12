use anyhow::Result;
use nistrs::prelude::*;
use std::io::{self, BufReader, Read, Write};

const BUFFER_SIZE: usize = 1024 * 64;         // 64 KB I/O chunk
const BLOCK_SIZE_BYTES: usize = 1024 * 1024;  // 1 MiB per test sequence

fn main() -> Result<()> {
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, io::stdin());

    let mut freq_pass = 0;
    let mut block_freq_pass = 0;
    let mut runs_pass = 0;
    let mut longest_run_pass = 0;
    let mut cumsum_forward_pass = 0;
    let mut cumsum_backward_pass = 0;
    let mut total_blocks = 0;

    eprintln!("--- NIST Streaming Test Suite (1 MiB blocks) ---");
    eprintln!("Processing stdin... (Ctrl+C to stop, or wait for EOF)\n");

    loop {
        // Read exactly one 1 MiB block
        let mut buffer = vec![0u8; BLOCK_SIZE_BYTES];
        let mut read_pos = 0;

        while read_pos < BLOCK_SIZE_BYTES {
            let n = reader.read(&mut buffer[read_pos..])?;
            if n == 0 {
                // EOF reached
                break;
            }
            read_pos += n;
        }

        if read_pos == 0 {
            // End of file, nothing more to read
            break;
        }

        if read_pos < BLOCK_SIZE_BYTES {
            // Partial block at the end – discard it
            eprintln!(
                "\nDiscarding {} leftover bytes (not enough for a full 1 MiB block)",
                read_pos
            );
            break;
        }

        // We have a full block – run all tests
        total_blocks += 1;
        let bits = BitsData::from_binary(buffer);

        // 1. Frequency
        let (pass, _) = frequency_test(&bits);
        if pass { freq_pass += 1; }

        // 2. Block Frequency
        match block_frequency_test(&bits, 128) {
            Ok((pass, _)) => if pass { block_freq_pass += 1; },
            Err(e) => eprintln!("\nBlock Frequency error on block {}: {}", total_blocks, e),
        }

        // 3. Runs
        let (pass, _) = runs_test(&bits);
        if pass { runs_pass += 1; }

        // 4. Longest Run of Ones
        match longest_run_of_ones_test(&bits) {
            Ok((pass, _)) => if pass { longest_run_pass += 1; },
            Err(e) => eprintln!("\nLongest Run error on block {}: {}", total_blocks, e),
        }

        // 5. Cumulative Sums (forward & backward)
        let results = cumulative_sums_test(&bits);
        if results.len() >= 2 {
            let (pass_f, _) = results[0];
            let (pass_b, _) = results[1];
            if pass_f { cumsum_forward_pass += 1; }
            if pass_b { cumsum_backward_pass += 1; }
        } else {
            eprintln!("\nCumulative Sums error on block {}", total_blocks);
        }

        // --- Live Progress Update (overwrites the same line) ---
        let n = total_blocks as f64;
        print!(
            "\rBlock {:>4} | Freq={:.3} Blk={:.3} Runs={:.3} Long={:.3} CumF={:.3} CumB={:.3}",
            total_blocks,
            freq_pass as f64 / n,
            block_freq_pass as f64 / n,
            runs_pass as f64 / n,
            longest_run_pass as f64 / n,
            cumsum_forward_pass as f64 / n,
            cumsum_backward_pass as f64 / n,
        );
        io::stdout().flush()?;
    }

    // --- Final Summary ---
    println!("\n\n--- Final Summary ---");
    if total_blocks == 0 {
        println!("No complete 1 MiB blocks were processed.");
        return Ok(());
    }

    let n = total_blocks as f64;
    println!("Total blocks processed: {}", total_blocks);
    println!(
        "Frequency                 : {:>3} / {:>3}   ({:.3})",
        freq_pass, total_blocks, freq_pass as f64 / n
    );
    println!(
        "Block Frequency (m=128)   : {:>3} / {:>3}   ({:.3})",
        block_freq_pass, total_blocks, block_freq_pass as f64 / n
    );
    println!(
        "Runs                      : {:>3} / {:>3}   ({:.3})",
        runs_pass, total_blocks, runs_pass as f64 / n
    );
    println!(
        "Longest Run of Ones       : {:>3} / {:>3}   ({:.3})",
        longest_run_pass, total_blocks, longest_run_pass as f64 / n
    );
    println!(
        "Cumulative Sums (forward) : {:>3} / {:>3}   ({:.3})",
        cumsum_forward_pass, total_blocks, cumsum_forward_pass as f64 / n
    );
    println!(
        "Cumulative Sums (backward): {:>3} / {:>3}   ({:.3})",
        cumsum_backward_pass, total_blocks, cumsum_backward_pass as f64 / n
    );

    Ok(())
}
