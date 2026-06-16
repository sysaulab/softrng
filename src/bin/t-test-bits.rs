use anyhow::{Context, Result};
use clap::Parser;
use memmap2::MmapMut;
use softrng::BUFFER_SIZE;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Read, Write};
use std::time::{Duration, Instant};
use uuid::Uuid;

use std::fs::File as StdFile;

#[derive(Parser)]
#[command(
    name = "t-test-bits",
    about = "Bit spectrum test for low/high halves of 64-bit words"
)]
struct Args {
    /// Bit width to test (1-64). Bits ≥ 34 use a memory‑mapped file automatically.
    #[arg(default_value = "32")]
    bits: u32,
    /// Maximum stage (0-10). Stage 0 completes after 1x space coverage.
    #[arg(short, long, default_value = "10")]
    max_stage: u32,
    /// Enable logging to CSV file (optional filename, default: bspec.log)
    #[arg(short, long, value_name = "FILE")]
    log: Option<String>,
    /// Results file (final stage scores)
    #[arg(short, long, default_value = "bspec32.txt")]
    results: String,
    /// Quiet mode: suppress stderr progress updates
    #[arg(short, long)]
    quiet: bool,
    /// Minimum seconds between progress updates (default 0.1 = ~10 FPS)
    #[arg(short, long, default_value = "0.1")]
    update_interval: f64,
}

enum BitmapBacking {
    Vec(Vec<u64>),
    Mmap(MmapMut),
}

impl BitmapBacking {
    fn as_mut_slice(&mut self) -> &mut [u64] {
        match self {
            BitmapBacking::Vec(v) => v.as_mut_slice(),
            BitmapBacking::Mmap(m) => {
                assert_eq!(m.len() % 8, 0);
                unsafe { std::slice::from_raw_parts_mut(m.as_mut_ptr() as *mut u64, m.len() / 8) }
            }
        }
    }
}

/// Adaptive sequential pre‑touch: walk the mmap in 4 MiB chunks,
/// time each chunk, and stop when a chunk takes ≥3× the baseline.
fn adaptive_prime(mmap: &mut MmapMut) {
    const PAGE_SIZE: usize = 4096;
    const CHUNK_PAGES: usize = 1000; // 4 MiB per chunk
    const WARMUP_CHUNKS: usize = 5; // skip first 5 chunks
    const BASELINE_SAMPLES: usize = 8; // use 8 chunks for baseline

    let total_pages = mmap.len() / PAGE_SIZE;
    let mut baseline_median: Option<Duration> = None;
    let mut chunk_times: Vec<Duration> = Vec::with_capacity(BASELINE_SAMPLES);
    let mut primed_pages = 0usize;

    for chunk_start in (0..total_pages).step_by(CHUNK_PAGES) {
        let end_page = (chunk_start + CHUNK_PAGES).min(total_pages);
        let start = Instant::now();

        // Touch one byte per page to force a page fault if not cached
        for page_offset in chunk_start..end_page {
            unsafe {
                std::ptr::read_volatile(&mmap[page_offset * PAGE_SIZE]);
            }
        }

        let elapsed = start.elapsed();
        primed_pages += end_page - chunk_start;

        // Skip the first few chunks to let the system stabilise
        if chunk_start / CHUNK_PAGES < WARMUP_CHUNKS {
            continue;
        }

        chunk_times.push(elapsed);
        if chunk_times.len() == BASELINE_SAMPLES {
            // Compute the median as a robust baseline
            chunk_times.sort();
            baseline_median = Some(chunk_times[chunk_times.len() / 2]);
        } else if let Some(base) = baseline_median {
            // If current chunk time exceeds 3× baseline (200% slowdown), stop
            if elapsed > base * 3 {
                if !primed_pages.is_power_of_two() { /* suppress compiler warning */ }
                eprintln!(
                    "\nAdaptive prime stopped after {:.1} MiB (page cache full)",
                    primed_pages as f64 * PAGE_SIZE as f64 / (1024.0 * 1024.0)
                );
                break;
            }
        }
    }
}

fn create_backing(bits: u32, quiet: bool) -> Result<(BitmapBacking, Option<std::path::PathBuf>)> {
    let space = 1u64 << bits;
    let len_bytes = ((space as usize + 63) / 64) * 8;

    if bits <= 33 {
        // Heap bitmap for small spaces
        Ok((BitmapBacking::Vec(vec![0u64; len_bytes / 8]), None))
    } else {
        // Mmap‑backed bitmap for large spaces
        let file_name = format!("t-test-bits-{}.mmap", Uuid::new_v4());
        let file_path = std::env::current_dir()?.join(&file_name);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .with_context(|| format!("Failed to create mmap file {}", file_path.display()))?;
        file.set_len(len_bytes as u64)
            .with_context(|| format!("Failed to allocate {} bytes", len_bytes))?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        // Hugepage hint on Linux (transparent on macOS)
        #[cfg(target_os = "linux")]
        unsafe {
            libc::madvise(
                mmap.as_mut_ptr() as *mut libc::c_void,
                mmap.len(),
                libc::MADV_HUGEPAGE,
            );
        }

        // Run adaptive prime
        adaptive_prime(&mut mmap);

        Ok((BitmapBacking::Mmap(mmap), Some(file_path)))
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    anyhow::ensure!(
        args.bits >= 1 && args.bits <= 64,
        "Bits must be 1..64, beware larger then 32 is slow."
    );
    let space = 1u64 << args.bits;
    let mask = space - 1;
    let bitset_size = (space as usize + 63) / 64;
    let mut bitset = vec![0u64; bitset_size];
    let mut unique_count = 0u64;
    let mut total_values = 0u64;

    // Optional log file
    let mut log: Option<File> = None;
    if let Some(log_path) = &args.log {
        let mut f = File::create(log_path)?;
        writeln!(f, "progress,score,seconds")?;
        log = Some(f);
    }

    let mut results = File::create(&args.results)?;

    let stdin = io::stdin().lock();
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, stdin);
    let mut byte_buffer = vec![0u8; BUFFER_SIZE];

    let start = Instant::now();
    let mut last_update = start;
    let mut stage = 0u32;
    let max_stage = args.max_stage.min(10);
    let stops = [
        0.0,
        0.9,
        0.99,
        0.999,
        0.9999,
        0.99999,
        0.999999,
        0.9999999,
        0.99999999,
        0.999999999,
        1.0,
    ];

    loop {
        let n = reader.read(&mut byte_buffer)?;
        if n == 0 {
            break;
        }

        let words = n / 8;
        let u64_slice =
            unsafe { std::slice::from_raw_parts(byte_buffer.as_ptr() as *const u64, words) };
        for &word in u64_slice {
            // Process low half
            let val = (word as u32) & (mask as u32);
            let v = val as usize;
            let word_idx = v / 64;
            let bit = v % 64;
            let mask_bit = 1u64 << bit;
            let was_zero = (bitset[word_idx] & mask_bit) == 0;
            bitset[word_idx] |= mask_bit;
            unique_count += was_zero as u64;

            // Process high half
            let val = ((word >> 32) as u32) & (mask as u32);
            let v = val as usize;
            let word_idx = v / 64;
            let bit = v % 64;
            let mask_bit = 1u64 << bit;
            let was_zero = (bitset[word_idx] & mask_bit) == 0;
            bitset[word_idx] |= mask_bit;
            unique_count += was_zero as u64;
        }
        total_values += (words * 2) as u64;

        // Expected unique count
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
            if stage > max_stage {
                break;
            }
        }

        // Throttled progress updates
        let now = Instant::now();
        if now.duration_since(last_update).as_secs_f64() >= args.update_interval {
            let progress = total_values as f64 / space as f64;
            let elapsed = now.duration_since(start).as_secs_f64();

            if let Some(ref mut f) = log {
                writeln!(f, "{:.12},{:.12},{:.3}", progress, score, elapsed)?;
            }
            if !args.quiet {
                eprint!(
                    "\rprogress: {:.2}%  score: {:.9}  stage: {}/{}",
                    progress * 100.0,
                    score,
                    stage,
                    max_stage
                );
                io::stderr().flush()?;
            }
            last_update = now;
        }
    }

    if !args.quiet {
        eprintln!();
    }
    Ok(())
}
