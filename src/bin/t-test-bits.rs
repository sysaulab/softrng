use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, Write, Read};
use std::time::Instant;
use softrng::BUFFER_SIZE;

#[derive(Parser)]
#[command(name = "t-test-bits", about = "Bit spectrum test for low/high halves of 64-bit words")]
struct Args {
    /// Bit width to test (1-32). Lower bits = smaller memory, faster convergence.
    #[arg(default_value = "32")]
    bits: u32,
    /// Maximum stage (0-10). Stage 0 completes after 1x space coverage.
    #[arg(short, long, default_value = "10")]
    max_stage: u32,
    /// Log file for detailed progress (CSV)
    #[arg(short, long, default_value = "bspec32.log")]
    log: String,
    /// Results file (final stage scores)
    #[arg(short, long, default_value = "bspec32.txt")]
    results: String,
    /// Quiet mode: suppress stderr progress updates
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    anyhow::ensure!(args.bits >= 1 && args.bits <= 32, "Bits must be 1..32");
    let space = 1u64 << args.bits;
    let mask = space - 1;
    let bitset_size = (space as usize + 63) / 64;
    let mut bitset = vec![0u64; bitset_size];
    let mut unique_count = 0u64;
    let mut total_values = 0u64;

    let mut log = File::create(&args.log)?;
    let mut results = File::create(&args.results)?;
    writeln!(log, "progress,score,seconds")?;

    let stdin = io::stdin().lock();
    let mut reader = io::BufReader::with_capacity(BUFFER_SIZE, stdin);
    let mut byte_buffer = vec![0u8; BUFFER_SIZE];

    let start = Instant::now();
    let mut stage_start = start;
    let mut stage = 0u32;
    let max_stage = args.max_stage.min(10);
    let stops = [0.0, 0.9, 0.99, 0.999, 0.9999, 0.99999, 0.999999, 0.9999999, 0.99999999, 0.999999999, 1.0];

    loop {
        let n = reader.read(&mut byte_buffer)?;
        if n == 0 {
            break;
        }

        // Process complete u64 words
        let words = n / 8;
        let u64_slice = unsafe {
            std::slice::from_raw_parts(byte_buffer.as_ptr() as *const u64, words)
        };
        for &word in u64_slice {
            // Mark low 32 bits
            let val = (word as u32) & (mask as u32);
            let v = val as usize;
            let word_idx = v / 64;
            let bit = v % 64;
            let mask_bit = 1u64 << bit;
            if bitset[word_idx] & mask_bit == 0 {
                bitset[word_idx] |= mask_bit;
                unique_count += 1;
            }

            // Mark high 32 bits
            let val = ((word >> 32) as u32) & (mask as u32);
            let v = val as usize;
            let word_idx = v / 64;
            let bit = v % 64;
            let mask_bit = 1u64 << bit;
            if bitset[word_idx] & mask_bit == 0 {
                bitset[word_idx] |= mask_bit;
                unique_count += 1;
            }
        }
        total_values += (words * 2) as u64;

        // Expected unique count (coupon collector formula)
        let expected = if total_values >= space {
            space as f64
        } else {
            space as f64 * (1.0 - (-(total_values as f64 / space as f64)).exp())
        };
        let score = if expected > 0.0 {
            (unique_count as f64 / expected).min(1.0)
        } else {
            0.0
        };

        // Stage progression
        let stage_completed = match stage {
            0 if total_values >= space => {
                let msg = format!("stage 0 : {:.9}", score);
                println!("\n{}", msg);
                writeln!(results, "{}", msg)?;
                true
            }
            s @ 1..=9 if score > stops[s as usize] => {
                let msg = format!("stage {} : {:.9}", s, total_values as f64 / space as f64);
                println!("\n{}", msg);
                writeln!(results, "{}", msg)?;
                true
            }
            10 if unique_count == space => {
                let msg = format!("stage 10 : {:.9}", total_values as f64 / space as f64);
                println!("\n{}", msg);
                writeln!(results, "{}", msg)?;
                true
            }
            _ => false,
        };

        if stage_completed {
            stage += 1;
            stage_start = Instant::now();
            if stage > max_stage {
                break;
            }
        }

        // Periodic logging (every second)
        let now = Instant::now();
        if now.duration_since(stage_start).as_secs() >= 1 {
            let progress = total_values as f64 / space as f64;
            writeln!(log, "{:.12},{:.12},{:.3}", progress, score, now.duration_since(start).as_secs_f64())?;
            if !args.quiet {
                eprint!("\rprogress: {:.2}%  score: {:.9}  stage: {}/{}", progress * 100.0, score, stage, max_stage);
            }
        }
    }

    if !args.quiet {
        eprintln!();
    }
    Ok(())
}