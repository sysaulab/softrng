use clap::{ArgAction, Parser};
use rayon::prelude::*;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

/// Streaming test harness for the SP800-90B non-IID min-entropy estimator.
///
/// Reads `SLICE_SIZE` bytes at a time from stdin, writes each slice to disk,
/// runs `ea_non_iid` on it, and reports min/max/mean/spread of the resulting
/// entropy bounds across `COUNT` slices.
///
/// Assessment is parallel across slices via rayon. Each child process is
/// pinned to a single OpenMP thread so parallelism lives at the slice level.
#[derive(Parser, Debug)]
#[command(
    name = "ea_stream_test",
    version,
    about = "Streaming SP800-90B non-IID min-entropy harness",
    long_about = None
)]
struct Cli {
    /// Number of slices to assess.
    #[arg(short = 'n', long, default_value_t = 10, value_name = "N")]
    count: usize,

    /// Bytes per slice read from stdin.
    #[arg(short = 's', long, default_value_t = 1_048_576, value_name = "BYTES")]
    slice_size: usize,

    /// Bits per symbol (1-8).
    #[arg(short = 'b', long, default_value_t = 8, value_name = "N")]
    bits_per_symbol: u8,

    /// Path to (or name of) the ea_non_iid helper.
    #[arg(long, default_value = "ea_non_iid", value_name = "PATH")]
    tool: PathBuf,

    /// Output directory (slice files, run.log).
    #[arg(long, default_value = "results", value_name = "DIR")]
    outdir: PathBuf,

    /// Optional CSV file to append the per-slice results to.
    #[arg(long, default_value = "results/results.csv", value_name = "PATH")]
    csv: Option<PathBuf>,

    /// Keep slice files after assessment.
    #[arg(long, action = ArgAction::SetTrue)]
    keep: bool,

    /// Use conditioned sequential dataset entropy estimate (-c).
    #[arg(short = 'c', long, action = ArgAction::SetTrue)]
    conditioned: bool,

    /// Forward -a (use all read bits for H_bitstring).
    #[arg(short = 'a', long, action = ArgAction::SetTrue)]
    all_bits: bool,

    /// Forward -t (truncate bitstring to 1,000,000 bits).
    #[arg(short = 't', long, action = ArgAction::SetTrue)]
    truncate: bool,

    /// Increase verbosity (repeatable); forwarded to the helper.
    #[arg(short = 'v', long, action = ArgAction::Count)]
    verbose: u8,

    /// Quiet mode: no per-slice progress, forward -q to helper.
    #[arg(short = 'q', long, action = ArgAction::SetTrue)]
    quiet: bool,

    /// Slices to assess in parallel (0 = auto = rayon default).
    #[arg(short = 'j', long, default_value_t = 0, value_name = "N")]
    jobs: usize,
}

#[derive(Debug)]
struct SliceOutcome {
    index: usize,
    n: usize,
    slice_path: PathBuf,
    h_orig: Option<f64>,
    h_bit: Option<f64>,
    h_min: Option<f64>,
    stdout: String,
    stderr: String,
    error: Option<String>,
}

fn assess_one(
    index: usize,
    buf: &[u8],
    tool_path: &Path,
    cli: &Cli,
    slicedir: &Path,
) -> SliceOutcome {
    let slice_path = slicedir.join(format!("slice_{index:04}"));

    let mk_err = |msg: String| SliceOutcome {
        index,
        n: buf.len(),
        slice_path: slice_path.clone(),
        h_orig: None,
        h_bit: None,
        h_min: None,
        stdout: String::new(),
        stderr: String::new(),
        error: Some(msg),
    };

    if let Err(e) = fs::write(&slice_path, buf) {
        return mk_err(format!("write {}: {e}", slice_path.display()));
    }

    let mut cmd = Command::new(tool_path);
    if cli.conditioned {
        cmd.arg("-c");
    } else {
        cmd.arg("-i");
    }
    if cli.truncate {
        cmd.arg("-t");
    } else if cli.all_bits {
        cmd.arg("-a");
    }
    for _ in 0..cli.verbose {
        cmd.arg("-v");
    }
    if cli.quiet {
        cmd.arg("-q");
    }
    cmd.arg(&slice_path);
    cmd.arg(cli.bits_per_symbol.to_string());
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Critical: one OpenMP thread per child. Parallelism lives at the slice level.
    cmd.env("OMP_NUM_THREADS", "1");
    cmd.env("OMP_DYNAMIC", "FALSE");

    match cmd.output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
            let h_orig = find_value(&stdout, "H_original");
            let h_bit = find_value(&stdout, "H_bitstring");
            let h_min = find_value(&stdout, "min(H_original").or_else(|| find_value(&stdout, "H_I"));
            SliceOutcome {
                index,
                n: buf.len(),
                slice_path,
                h_orig,
                h_bit,
                h_min,
                stdout,
                stderr,
                error: None,
            }
        }
        Err(e) => mk_err(format!("launch {}: {e}", tool_path.display())),
    }
}

fn main() {
    let cli = Cli::parse();

    if !(1..=8).contains(&cli.bits_per_symbol) {
        eprintln!("error: --bits-per-symbol must be between 1 and 8");
        std::process::exit(2);
    }
    if cli.count == 0 {
        eprintln!("error: --count must be >= 1");
        std::process::exit(2);
    }
    if cli.slice_size == 0 {
        eprintln!("error: --slice-size must be >= 1");
        std::process::exit(2);
    }

    // ---- locate helper ---------------------------------------------------
    let tool_path = match resolve_tool(&cli.tool) {
        Some(p) => p,
        None => {
            eprintln!(
                "error: could not find helper program `{}`.",
                cli.tool.display()
            );
            eprintln!(
                "       Pass `--tool <PATH>` or make sure `ea_non_iid` is on PATH."
            );
            std::process::exit(2);
        }
    };

    // ---- prepare output dirs --------------------------------------------
    if let Err(e) = fs::create_dir_all(&cli.outdir) {
        eprintln!(
            "error: cannot create outdir `{}`: {e}",
            cli.outdir.display()
        );
        std::process::exit(1);
    }
    let slicedir = cli.outdir.join("slices");
    if let Err(e) = fs::create_dir_all(&slicedir) {
        eprintln!("error: cannot create `{}`: {e}", slicedir.display());
        std::process::exit(1);
    }

    // ---- open CSV (optional) --------------------------------------------
    let mut csv_file = match &cli.csv {
        Some(p) => {
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }
            let is_new = !p.exists();
            let mut f = match OpenOptions::new().create(true).append(true).open(p) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("error: cannot open CSV `{}`: {e}", p.display());
                    std::process::exit(1);
                }
            };
            if is_new {
                let _ = writeln!(
                    f,
                    "slice,file,size,H_original,H_bitstring,min_bound"
                );
            }
            Some(f)
        }
        None => None,
    };

    // ---- open run log ---------------------------------------------------
    let log_path = cli.outdir.join("run.log");
    let mut log_file = match File::create(&log_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: cannot create log `{}`: {e}", log_path.display());
            std::process::exit(1);
        }
    };

    // ---- read all slices into memory ------------------------------------
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut slices: Vec<(usize, Vec<u8>)> = Vec::with_capacity(cli.count);

    for i in 1..=cli.count {
        let mut buf = vec![0u8; cli.slice_size];
        match read_exact_or_eof(&mut stdin_lock, &mut buf) {
            Ok(0) => {
                if i == 1 {
                    eprintln!("error: stdin is empty");
                    std::process::exit(1);
                }
                if !cli.quiet {
                    eprintln!("[stopped] EOF at slice {i}: input exhausted");
                }
                break;
            }
            Ok(n) => {
                if n < cli.slice_size {
                    eprintln!(
                        "warning: slice {i} got only {n} bytes (expected {}); assessing anyway",
                        cli.slice_size
                    );
                }
                buf.truncate(n);
                slices.push((i, buf));
            }
            Err(e) => {
                eprintln!("error: stdin read failed at slice {i}: {e}");
                break;
            }
        }
    }
    drop(stdin_lock);

    // ---- parallel assessment --------------------------------------------
    let jobs = if cli.jobs == 0 {
        rayon::current_num_threads()
    } else {
        cli.jobs
    };
    if !cli.quiet {
        eprintln!(
            "[ea_stream_test] assessing {} slice(s) with {} worker(s)",
            slices.len(),
            jobs
        );
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .expect("failed to build rayon pool");

    let started = Instant::now();
    let outcomes: Vec<SliceOutcome> = pool.install(|| {
        slices
            .par_iter()
            .map(|(i, buf)| assess_one(*i, buf, &tool_path, &cli, &slicedir))
            .collect()
    });
    let elapsed = started.elapsed();

    // ---- sequential reporting (order preserved by par_iter/collect) -----
    let mut assessed: usize = 0;
    let mut values: Vec<f64> = Vec::with_capacity(outcomes.len());

    for o in &outcomes {
        if let Some(msg) = &o.error {
            eprintln!("error at slice {}: {msg}", o.index);
            if !cli.keep {
                let _ = fs::remove_file(&o.slice_path);
            }
            continue;
        }

        let _ = writeln!(
            log_file,
            "==== slice {} ({}) ====",
            o.index,
            o.slice_path.display()
        );
        let _ = log_file.write_all(o.stdout.as_bytes());
        if !o.stderr.is_empty() {
            let _ = log_file.write_all(o.stderr.as_bytes());
        }

        if let Some(f) = csv_file.as_mut() {
            let fmt = |v: Option<f64>| {
                v.map(|x| format!("{x:.10}"))
                    .unwrap_or_else(|| "NA".to_string())
            };
            let _ = writeln!(
                f,
                "{},{},{},{},{},{}",
                o.index,
                o.slice_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("?"),
                o.n,
                fmt(o.h_orig),
                fmt(o.h_bit),
                fmt(o.h_min),
            );
        }

        if !cli.quiet {
            let v = |x: Option<f64>| {
                x.map(|v| format!("{v:.6}"))
                    .unwrap_or_else(|| "NA".to_string())
            };
            println!(
                "[{}/{}] {} ({} bytes) ... H_orig={} H_bit={} min={}",
                o.index,
                cli.count,
                o.slice_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("?"),
                o.n,
                v(o.h_orig),
                v(o.h_bit),
                v(o.h_min),
            );
        }

        assessed += 1;
        if let Some(v) = o.h_min {
            values.push(v);
        }

        if !cli.keep {
            let _ = fs::remove_file(&o.slice_path);
        }
    }

    // ---- flush -----------------------------------------------------------
    if let Some(f) = csv_file.as_mut() {
        let _ = f.flush();
    }
    let _ = log_file.flush();

    // ---- summary ---------------------------------------------------------
    println!();
    println!("=== summary ===");
    println!("slices assessed: {}", assessed);
    println!("elapsed:         {:.2?}", elapsed);

    if values.is_empty() {
        println!("valid results:   0");
        println!("no valid entropy results were parsed from the helper output");
    } else {
        let n = values.len();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = values.iter().sum();
        let mean = sum / n as f64;
        println!("valid results:   {}", n);
        println!("min:             {:.6}", min);
        println!("max:             {:.6}", max);
        println!("mean:            {:.6}", mean);
        println!("spread (max-min): {:.6}", max - min);
    }

    println!();
    println!("log:  {}", log_path.display());
    if let Some(p) = &cli.csv {
        println!("csv:  {}", p.display());
    }
    if cli.keep {
        println!("slices kept in {}", slicedir.display());
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Read until the buffer is full or EOF. Returns number of bytes read.
fn read_exact_or_eof<R: Read>(r: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    let mut total = 0;
    while total < buf.len() {
        match r.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(total)
}

/// Locate the helper program. Supports either a filesystem path (must exist)
/// or a bare name that will be searched for on `PATH`.
fn resolve_tool(p: &Path) -> Option<PathBuf> {
    if p.is_absolute() || p.components().count() > 1 {
        return p.is_file().then(|| p.to_path_buf());
    }
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let cand = dir.join(p);
            if cand.is_file() {
                return Some(cand);
            }
            #[cfg(windows)]
            {
                let exe = cand.with_extension("exe");
                if exe.is_file() {
                    return Some(exe);
                }
            }
        }
    }
    // Fallback: check cwd (matches the `./ea_non_iid` default from the shell script).
    if p.is_file() {
        return Some(p.to_path_buf());
    }
    None
}

/// Find the floating-point value following `key` on its line, e.g.
/// `H_original: 7.616427` -> 7.616427, or
/// `min(H_original, 8 X H_bitstring): 7.616427` -> 7.616427.
fn find_value(output: &str, key: &str) -> Option<f64> {
    for line in output.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(key) {
            if let Some(colon) = rest.find(':') {
                let after = rest[colon + 1..].trim();
                if let Some(tok) = after.split_whitespace().next() {
                    if let Ok(v) = tok.parse::<f64>() {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}