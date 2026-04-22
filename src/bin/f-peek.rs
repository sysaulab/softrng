use anyhow::Result;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use softrng::{HEX_CHARS, BUFFER_SIZE};

const DISPLAY_FPS: u64 = 30;                              // UI refresh rate
const DISPLAY_INTERVAL: Duration = Duration::from_micros(1_000_000 / DISPLAY_FPS);

/// Write a size to a writer in human-readable form (binary units).
fn write_size(w: &mut impl Write, bytes: u64) -> io::Result<()> {
    const UNITS: &[(&[u8], u64)] = &[
        (b"Eb", 1u64 << 60),
        (b"Pb", 1u64 << 50),
        (b"Tb", 1u64 << 40),
        (b"Gb", 1u64 << 30),
        (b"Mb", 1u64 << 20),
        (b"Kb", 1u64 << 10),
        (b"b", 1),
    ];

    for &(suffix, threshold) in UNITS {
        if bytes >= threshold {
            let main = bytes >> threshold.trailing_zeros();
            if threshold == 1 {
                write!(w, "{}{}", main, std::str::from_utf8(suffix).unwrap())?;
            } else {
                let rem = (bytes & (threshold - 1)) * 100 / threshold;
                write!(w, "{}.{:02}{}", main, rem, std::str::from_utf8(suffix).unwrap())?;
            }
            return Ok(());
        }
    }
    write!(w, "0b")
}

/// Write hex representation of up to 8 bytes (stack‑allocated).
fn write_hex_preview(w: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    let mut hex = [0u8; 16];
    for (i, &b) in bytes.iter().enumerate() {
        hex[i * 2] = HEX_CHARS[(b >> 4) as usize];
        hex[i * 2 + 1] = HEX_CHARS[(b & 0x0f) as usize];
    }
    w.write_all(&hex[..bytes.len() * 2])
}

fn main() -> Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut total_bytes = 0u64;

    // Shared atomic timestamp (nanoseconds since start)
    let now_ns = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    // Spawn background thread that updates the atomic timestamp at ~30 FPS
    let now_ns_clone = Arc::clone(&now_ns);
    thread::spawn(move || {
        loop {
            let elapsed = start.elapsed().as_nanos() as u64;
            now_ns_clone.store(elapsed, Ordering::Relaxed);
            thread::sleep(DISPLAY_INTERVAL);
        }
    });

//    let start_time = Instant::now();
    let mut last_display_ns = 0u64;
    let mut first_chunk = true;

    loop {
        let n = stdin.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        stdout.write_all(&buffer[..n])?;
        total_bytes += n as u64;

        // Cheap atomic read – no syscall
        let current_ns = now_ns.load(Ordering::Relaxed);

        // Update display if enough time has passed, or on the very first chunk
        if first_chunk || current_ns.saturating_sub(last_display_ns) >= DISPLAY_INTERVAL.as_nanos() as u64 {
            let preview = if n >= 8 { &buffer[..8] } else { &buffer[..n] };
            stderr.write_all(b"\r[")?;
            write_hex_preview(&mut stderr, preview)?;
            stderr.write_all(b"] (")?;
            write_size(&mut stderr, total_bytes)?;
            stderr.write_all(b") ")?;

            let elapsed_secs = current_ns as f64 / 1_000_000_000.0;
            let speed = if elapsed_secs > 0.0 {
                (total_bytes as f64 / elapsed_secs) as u64
            } else {
                0
            };
            write_size(&mut stderr, speed)?;
            stderr.write_all(b"/sec   ")?;
            stderr.flush()?;

            last_display_ns = current_ns;
            first_chunk = false;
        }
    }

    // Final newline
    eprintln!();
    Ok(())
}