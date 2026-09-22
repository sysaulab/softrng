use std::io::{self, BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Mode {
    /// Time sched_yield() directly with a clock.
    Clock,
    /// Measure how much a peer thread advanced a shared counter during sched_yield().
    Counter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Extract {
    /// Flip a bit when the yield delta is non-zero. Bit position cycles 0..63.
    Nonzero,
    /// Shift the low bit of the delta into a rotating accumulator.
    Lowbit,
}

#[derive(Parser, Debug)]
#[command(
    name = "seedy64-yield",
    version,
    about = "Yield-only entropy prototype",
    long_about = "Measures scheduler jitter from sched_yield() alone, with no \
                  mixing ring and no timer in the emission path. Emits raw \
                  64-bit words to stdout."
)]
struct Args {
    #[arg(long, value_enum, default_value_t = Mode::Clock)]
    mode: Mode,

    #[arg(long, value_enum, default_value_t = Extract::Nonzero)]
    extract: Extract,

    /// Use rdtsc (x86_64) or cntvct_el0 (aarch64) instead of clock_gettime.
    /// Falls back to the portable path on other architectures.
    #[arg(long, default_value_t = false)]
    optimized: bool,

    /// Number of yields folded into each 64-bit output word.
    #[arg(long, default_value_t = 512)]
    pack: u32,

    /// Number of 64-bit words to emit. 0 = unlimited (until stdout closes).
    #[arg(long, default_value_t = 0)]
    count: u64,

    /// Warmup duration in nanoseconds.
    #[arg(long, default_value_t = 1_000_000)]
    warmup_ns: u64,

    /// Run a peer thread so that sched_yield() has someone to yield to.
    /// In counter mode this is the thread that advances the counter.
    #[arg(long, default_value_t = true)]
    peer: bool,
}

// ----------------------------------------------------------------
// Timing backends
// ----------------------------------------------------------------

#[inline(always)]
fn now_portable() -> u64 {
    // clock_gettime(CLOCK_MONOTONIC) via vDSO. ~20-25 ns per call on modern Linux.
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe {
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
    }
    (ts.tv_sec as u64)
        .wrapping_mul(1_000_000_000)
        .wrapping_add(ts.tv_nsec as u64)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn now_optimized() -> u64 {
    // rdtsc. Safe here because sched_yield() is a syscall and acts as a
    // barrier; the CPU cannot reorder the timer reads across it in a way
    // that matters for this measurement.
    unsafe { core::arch::x86_64::_rdtsc() }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn now_optimized() -> u64 {
    let t: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) t, options(nostack, nomem));
    }
    t
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline(always)]
fn now_optimized() -> u64 {
    now_portable()
}

// ----------------------------------------------------------------
// Peer thread
// ----------------------------------------------------------------

fn spawn_peer(
    run: Arc<AtomicBool>,
    counter: Option<Arc<AtomicU64>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while run.load(Ordering::Relaxed) {
            match &counter {
                // Counter mode: tight increment loop.
                Some(c) => {
                    c.fetch_add(1, Ordering::Relaxed);
                }
                // Clock mode: busy spin with periodic yields, so that a
                // yield from the main thread has someone to hand the CPU
                // to regardless of core placement.
                None => {
                    for _ in 0..64 {
                        std::hint::spin_loop();
                    }
                    unsafe {
                        libc::sched_yield();
                    }
                }
            }
        }
    })
}

// ----------------------------------------------------------------
// Emit loops
// ----------------------------------------------------------------

fn run_clock<F: Fn() -> u64>(
    now: F,
    args: &Args,
    out: &mut impl Write,
) -> io::Result<()> {
    let pack = args.pack.max(1) as usize;
    let mut written: u64 = 0;

    loop {
        let mut acc: u64 = 0;

        match args.extract {
            Extract::Nonzero => {
                for i in 0..pack {
                    let t0 = now();
                    unsafe {
                        libc::sched_yield();
                    }
                    let t1 = now();
                    if t1 != t0 {
                        acc ^= 1u64 << (i & 63);
                    }
                }
            }
            Extract::Lowbit => {
                for _ in 0..pack {
                    let t0 = now();
                    unsafe {
                        libc::sched_yield();
                    }
                    let t1 = now();
                    acc = acc.rotate_left(1) | (t1.wrapping_sub(t0) & 1);
                }
            }
        }

        out.write_all(&acc.to_le_bytes())?;
        written += 1;
        if args.count > 0 && written >= args.count {
            break;
        }
    }
    Ok(())
}

fn run_counter(
    args: &Args,
    out: &mut impl Write,
    shared: &AtomicU64,
) -> io::Result<()> {
    let pack = args.pack.max(1) as usize;
    let mut written: u64 = 0;

    loop {
        let mut acc: u64 = 0;

        match args.extract {
            Extract::Nonzero => {
                for i in 0..pack {
                    let c0 = shared.load(Ordering::Relaxed);
                    unsafe {
                        libc::sched_yield();
                    }
                    let c1 = shared.load(Ordering::Relaxed);
                    if c1 != c0 {
                        acc ^= 1u64 << (i & 63);
                    }
                }
            }
            Extract::Lowbit => {
                for _ in 0..pack {
                    let c0 = shared.load(Ordering::Relaxed);
                    unsafe {
                        libc::sched_yield();
                    }
                    let c1 = shared.load(Ordering::Relaxed);
                    acc = acc.rotate_left(1) | (c1.wrapping_sub(c0) & 1);
                }
            }
        }

        out.write_all(&acc.to_le_bytes())?;
        written += 1;
        if args.count > 0 && written >= args.count {
            break;
        }
    }
    Ok(())
}

// ----------------------------------------------------------------
// main
// ----------------------------------------------------------------

fn main() -> io::Result<()> {
    let args = Args::parse();

    let optimized_supported = cfg!(any(
        target_arch = "x86_64",
        target_arch = "aarch64"
    ));
    let use_optimized = args.optimized && optimized_supported;
    if args.optimized && !optimized_supported {
        eprintln!(
            "note: --optimized requested but not supported on this architecture; \
             using portable clock_gettime path"
        );
    }
    let now: fn() -> u64 = if use_optimized {
        now_optimized
    } else {
        now_portable
    };

    let run = Arc::new(AtomicBool::new(true));
    let shared = Arc::new(AtomicU64::new(0));

    let peer_handle = if args.peer {
        let counter = if args.mode == Mode::Counter {
            Some(Arc::clone(&shared))
        } else {
            None
        };
        Some(spawn_peer(Arc::clone(&run), counter))
    } else {
        None
    };

    thread::sleep(Duration::from_nanos(args.warmup_ns));

    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(64 * 1024, stdout.lock());

    let result = match args.mode {
        Mode::Clock => run_clock(now, &args, &mut out),
        Mode::Counter => run_counter(&args, &mut out, &shared),
    };

    let _ = out.flush();

    run.store(false, Ordering::Relaxed);
    if let Some(h) = peer_handle {
        let _ = h.join();
    }

    result
}