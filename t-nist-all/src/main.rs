use anyhow::Result;
use clap::Parser;
use entropy::nist;
use entropy::rng::Rng as EntropyRng;
use rand::RngCore;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

// -----------------------------------------------------------------------------
// Command-line arguments
// -----------------------------------------------------------------------------

#[derive(Parser)]
#[command(author, version, about = "Full NIST SP 800-22 streaming tester (all sub-tests)")]
struct Args {
    /// Number of bytes per block (must be at least 125,000)
    #[arg(short, long, default_value_t = 1_048_576 * 2)] // 4 MiB
    block_size: usize,

    /// Number of blocks to test (0 = run until EOF)
    #[arg(short, long, default_value_t = 10)]
    runs: usize,

    /// Path to CSV file for detailed logging (required)
    #[arg(short, long)]
    csv: PathBuf,
}

// -----------------------------------------------------------------------------
// Custom RNG – feeds bytes from a fixed buffer
// -----------------------------------------------------------------------------

struct BufferRng {
    data: Vec<u8>,
    pos: usize,
}

impl RngCore for BufferRng {
    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.fill_bytes(&mut buf);
        u32::from_le_bytes(buf)
    }

    fn next_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf);
        u64::from_le_bytes(buf)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let start = self.pos;
        let end = self.pos + dest.len();
        if end > self.data.len() {
            panic!(
                "Not enough data: requested {} bytes, only {} remaining",
                dest.len(),
                self.data.len() - self.pos
            );
        }
        dest.copy_from_slice(&self.data[start..end]);
        self.pos = end;
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl EntropyRng for BufferRng {
    fn next_u32(&mut self) -> u32 {
        <Self as RngCore>::next_u32(self)
    }
}

// -----------------------------------------------------------------------------
// Main
// -----------------------------------------------------------------------------

fn main() -> Result<()> {
    let args = Args::parse();

    const MIN_BYTES: usize = 125_000;
    if args.block_size < MIN_BYTES {
        anyhow::bail!(
            "block_size ({}) is below minimum {} bytes",
            args.block_size,
            MIN_BYTES
        );
    }

    // Prepare CSV output
    let csv_file = File::create(&args.csv)?;
    let mut csv_writer = BufWriter::new(csv_file);
    writeln!(csv_writer, "Block,TestName,PValue,Pass")?;

    let mut reader = BufReader::with_capacity(64 * 1024, io::stdin());

    // Accumulate per-test statistics: (passes, total_runs)
    let mut test_stats: HashMap<&'static str, (usize, usize)> = HashMap::new();
    let mut total_blocks = 0;

    eprintln!("--- Full NIST Streaming Suite ---");
    eprintln!("Block size: {} bytes", args.block_size);
    eprintln!("CSV logging to: {}", args.csv.display());
    if args.runs == 0 {
        eprintln!("Processing stdin until EOF... (Ctrl+C to stop)\n");
    } else {
        eprintln!("Processing up to {} blocks...\n", args.runs);
    }

    loop {
        if args.runs > 0 && total_blocks >= args.runs {
            break;
        }

        // Read exactly one block
        let mut buffer = vec![0u8; args.block_size];
        let mut read_pos = 0;

        while read_pos < args.block_size {
            let n = reader.read(&mut buffer[read_pos..])?;
            if n == 0 {
                break;
            }
            read_pos += n;
        }

        if read_pos == 0 {
            break;
        }
        if read_pos < args.block_size {
            eprintln!(
                "\nDiscarding {} leftover bytes (incomplete block)",
                read_pos
            );
            break;
        }

        total_blocks += 1;
        let total_bits = args.block_size * 8;

        // Run the FULL NIST suite on this block
        let mut rng = BufferRng { data: buffer, pos: 0 };
        let results = nist::run_all(&mut rng, total_bits);

        let block_test_count = results.len();

        // Update stats and write CSV
        for res in results {
            let pass = res.p_value >= 0.01;
            let entry = test_stats.entry(res.name).or_insert((0, 0));
            entry.1 += 1; // total runs
            if pass {
                entry.0 += 1; // passes
            }

            writeln!(
                csv_writer,
                "{},{},{:.12},{}",
                total_blocks,
                res.name,
                res.p_value,
                if pass { "PASS" } else { "FAIL" }
            )?;
        }
        csv_writer.flush()?;

        // Compute the *average pass rate* across all test results so far.
        // This includes all tests and all blocks processed up to now.
        let total_passes: usize = test_stats.values().map(|(p, _)| p).sum();
        let total_runs: usize = test_stats.values().map(|(_, t)| t).sum();
        let avg_rate = if total_runs > 0 {
            (total_passes as f64 / total_runs as f64) * 100.0
        } else {
            0.0
        };

        // Live display: block number, number of tests in this block, and the running average
        print!(
            "\rBlock {:>4} ({:>3} tests) PASS rate={:>6.2}%",
            total_blocks,
            block_test_count,
            avg_rate
        );
        io::stdout().flush()?;
    }

    println!(); // final newline

    // --- Final Summary ---
    println!("\n--- Final Summary ---");
    if total_blocks == 0 {
        println!("No complete blocks processed.");
        return Ok(());
    }

    // Overall average
    let total_passes: usize = test_stats.values().map(|(p, _)| p).sum();
    let total_runs: usize = test_stats.values().map(|(_, t)| t).sum();
    let overall_avg = if total_runs > 0 {
        (total_passes as f64 / total_runs as f64) * 100.0
    } else {
        0.0
    };

    println!("Total blocks processed: {}", total_blocks);
    println!("Total test runs (tests × blocks): {}", total_runs);
    println!("Overall average pass rate: {:.2}%", overall_avg);

    // ----- Ranking: worst to best -----
    // Collect all test names and their pass rates
    let mut ranked: Vec<(&'static str, f64)> = Vec::with_capacity(test_stats.len());
    for (name, (passes, total)) in test_stats.iter() {
        let rate = if *total > 0 {
            (*passes as f64 / *total as f64) * 100.0
        } else {
            0.0
        };
        ranked.push((*name, rate));
    }

    // Sort by pass rate ascending (worst first)
    ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\n--- Test Ranking (worst to best by pass rate) ---");
    println!("{:<50} | {:>10}", "Test Name", "Pass Rate");
    println!("{}-|-{}", "-".repeat(50), "-".repeat(12));
    for (name, rate) in ranked {
        println!("{:<50} | {:>9.2}%", name, rate);
    }

    println!("\nDetailed CSV written to: {}", args.csv.display());

    Ok(())
}