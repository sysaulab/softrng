use anyhow::{Context, Result};
use std::io::{self, Read, Write};
use std::sync::mpsc::{self, SyncSender};
use std::sync::Arc;
use std::thread;

// ────────────────────────────────────────
// PQXO64 core constants
// ────────────────────────────────────────
const PRIME_STEP: u64 = 7776210437768060567;
const TABLE_ENTRIES: usize = 65536;
const NUM_TABLES: usize = 4;
const MAP_SIZE: usize = TABLE_ENTRIES * NUM_TABLES * 8; // 2 MiB
const DEFAULT_THREADS: usize = 0; // 0 means "auto"
const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024; // 1 MiB

// ────────────────────────────────────────
// The shared lookup table
// ────────────────────────────────────────
type Table = Arc<[u64; TABLE_ENTRIES * NUM_TABLES]>;

/// Create the shared table from a 2 MiB seed (native‑endian u64).
fn make_table(seed: &[u8]) -> Result<Table> {
    if seed.len() != MAP_SIZE {
        anyhow::bail!("seed must be exactly 2 MiB");
    }
    let mut table = Box::new([0u64; TABLE_ENTRIES * NUM_TABLES]);
    unsafe {
        std::ptr::copy_nonoverlapping(
            seed.as_ptr(),
            table.as_mut_ptr() as *mut u8,
            MAP_SIZE,
        );
    }
    Ok(Arc::new(*table))
}

/// Core mixing: counter → four table lookups → XOR.
#[inline]
fn read_chunk(table: &[u64], counter: u64) -> u64 {
    let i0 = (counter & 0xFFFF) as usize;
    let i1 = ((counter >> 16) & 0xFFFF) as usize;
    let i2 = ((counter >> 32) & 0xFFFF) as usize;
    let i3 = ((counter >> 48) & 0xFFFF) as usize;

    table[i0]
        ^ table[TABLE_ENTRIES + i1]
        ^ table[2 * TABLE_ENTRIES + i2]
        ^ table[3 * TABLE_ENTRIES + i3]
}

// ────────────────────────────────────────
// Worker thread logic
// ────────────────────────────────────────
fn worker(
    table: Table,
    start_counter: u64,
    stride: u64,
    chunk_size: usize,
    tx: SyncSender<Vec<u8>>,
) {
    assert!(chunk_size % 8 == 0);
    let words_per_chunk = chunk_size / 8;
    let mut ctr = start_counter;

    loop {
        let mut chunk = vec![0u8; chunk_size];
        let words = bytemuck::cast_slice_mut(&mut chunk);

        for word in words.iter_mut() {
            // Fix: convert Arc to slice reference via as_ref()
            *word = read_chunk(table.as_ref(), ctr).to_ne_bytes();
            ctr = ctr.wrapping_add(stride);
        }

        if tx.send(chunk).is_err() {
            break;
        }
    }
}

// ────────────────────────────────────────
// Main entry point
// ────────────────────────────────────────
fn main() -> Result<()> {
    // Parse optional thread count from env or arg (simple).
    let threads: usize = std::env::var("PQXO64_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_THREADS);

    let cpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let n_threads = if threads == 0 {
        std::cmp::max(1, cpus / 2)
    } else {
        threads.clamp(1, cpus)
    };

    // Read 2 MiB seed from stdin.
    let mut seed = vec![0u8; MAP_SIZE];
    io::stdin()
        .lock()
        .read_exact(&mut seed)
        .context("Failed to read 2 MiB seed from stdin")?;

    let table = make_table(&seed)?;

    // ──────────────────────────────────
    // Set up the producer‑consumer pipeline
    // ──────────────────────────────────
    let chunk_size = DEFAULT_CHUNK_SIZE;
    let stride = n_threads as u64 * PRIME_STEP;

    // Bounded channel: prevents memory blowup if generation is faster than output.
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(n_threads * 2);

    // Spawn worker threads.
    for t in 0..n_threads {
        let table = Arc::clone(&table);
        let tx = tx.clone();
        let start = t as u64 * PRIME_STEP; // distinct starting counter
        thread::spawn(move || worker(table, start, stride, chunk_size, tx));
    }
    drop(tx); // main doesn't need the sender

    // ──────────────────────────────────
    // Writer thread (the main thread here)
    // ──────────────────────────────────
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    while let Ok(chunk) = rx.recv() {
        handle.write_all(&chunk)?;
    }

    Ok(())
}
