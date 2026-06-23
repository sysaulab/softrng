use clap::Parser;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Profiles OS sleep timer granularity by measuring a decaying sequence of delays.
#[derive(Parser, Debug)]
#[command(name = "timer-profile")]
struct Args {
    /// Number of trials per delay step [10..=1000]
    #[arg(short = 'n', long, default_value = "100", value_name = "N")]
    iterations: usize,

    /// Starting delay in milliseconds [1..=1000]
    #[arg(short = 's', long, default_value = "100", value_name = "MS")]
    start_delay_ms: u32,

    /// How much to multiply the delay after a passing step (0.1 .. 0.9)
    #[arg(short = 'f', long, default_value = "0.5", value_name = "FACTOR")]
    progress_factor: f64,

    /// Optional CSV output file
    #[arg(long, value_name = "FILE")]
    csv: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    // Validate ranges
    if args.iterations < 10 || args.iterations > 1000 {
        eprintln!("iterations must be between 10 and 1000");
        std::process::exit(1);
    }
    if args.start_delay_ms < 1 || args.start_delay_ms > 1000 {
        eprintln!("start-delay-ms must be between 1 and 1000");
        std::process::exit(1);
    }
    if args.progress_factor < 0.1 || args.progress_factor > 0.9 {
        eprintln!("progress-factor must be between 0.1 and 0.9");
        std::process::exit(1);
    }

    let mut csv_writer: Option<Box<dyn Write>> = if let Some(ref path) = args.csv {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        let mut w: Box<dyn Write> = Box::new(file);
        writeln!(w, "D_ms,mean_T_ms,RMSE_ms,score")?;
        Some(w)
    } else {
        None
    };

    // Warm‑up
    std::thread::sleep(Duration::from_millis(100));

    let mut last_passing_d: Option<f64> = None;
    let mut delay_secs = args.start_delay_ms as f64 / 1_000.0;
    let min_d = 1e-9; // 1 ns in seconds

    println!("D_ms,mean_T_ms,RMSE_ms,score");
    while delay_secs >= min_d {
        let d_duration = Duration::from_secs_f64(delay_secs);
        let n = args.iterations;
        let mut errors_sq = 0.0f64;
        let mut sum = 0.0f64;
        for _ in 0..n {
            let start = Instant::now();
            std::thread::sleep(d_duration);
            let elapsed = start.elapsed().as_secs_f64();
            let err = elapsed - delay_secs;
            errors_sq += err * err;
            sum += elapsed;
        }
        let mean = sum / n as f64;
        let rmse = (errors_sq / n as f64).sqrt();
        let score = (1.0 - rmse / delay_secs).max(0.0);

        let d_ms = delay_secs * 1_000.0;
        let mean_ms = mean * 1_000.0;
        let rmse_ms = rmse * 1_000.0;
        println!("{:.6},{:.6},{:.6},{:.6}", d_ms, mean_ms, rmse_ms, score);
        if let Some(ref mut w) = csv_writer {
            writeln!(w, "{:.6},{:.6},{:.6},{:.6}", d_ms, mean_ms, rmse_ms, score)?;
            w.flush()?;
        }

        if score >= 0.1 {
            last_passing_d = Some(delay_secs);
        } else {
            break; // stop immediately on first failure
        }

        delay_secs *= args.progress_factor;
    }

    match last_passing_d {
        Some(d) => println!(
            "Granularity threshold: {:.3} ns (last delay with score >= 0.1: {:.3} ms)",
            d * 1_000_000_000.0,
            d * 1_000.0
        ),
        None => println!("No delay met the score >= 0.1 threshold."),
    }

    Ok(())
}