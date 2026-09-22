use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use speedy64::{Config, Generator};

/// speedy64 — userspace entropy source.
///
/// Writes raw 64-bit words to stdout: native-endian by default, no framing,
/// no delimiters. Exits when stdout closes.
///
/// NOT FOR SECURITY USE. The output is statistically clean and
/// cryptographically insecure. Do not use it for keys, tokens, nonces, IVs,
/// passwords, or session IDs.
#[derive(Parser, Debug)]
#[command(
    name = "s-speedy64",
    version,
    about = "Timing-jitter harvesting and statistical mixing in one pass.",
    long_about = None,
    after_help = "EXAMPLES:\n  \
        s-speedy64 > stream.bin\n  \
        s-speedy64 3 1000000 > stream.bin\n  \
        s-speedy64 | head -c 1000000 > slice.bin\n\n\
        The first positional argument is SKIP (default 1), the second is INIT_NS (default 1000000)."
)]
struct Args {
    /// Number of full mixing passes between emitted words.
    ///
    /// Defaults to 1. Larger values reduce the emission rate without
    /// replacing the scheduler event that skip=1 provides.
    #[arg(value_name = "SKIP", default_value_t = 1)]
    skip: u64,

    /// Warm-up duration in nanoseconds before the first emit.
    ///
    /// Mixing threads run during this window. One millisecond is enough on
    /// bare metal; raise it on loaded or virtualised hosts.
    #[arg(value_name = "INIT_NS", default_value_t = 1_000_000)]
    init_ns: u64,

    /// Write byteswapped (big-endian) words instead of native-endian.
    #[arg(long, default_value_t = false)]
    big_endian: bool,

    /// Disable the pre-emit scheduler yield.
    ///
    /// The yield is the entropy mechanism. Disabling it produces output
    /// that fails PractRand. Provided only for the negative-control
    /// experiments described in the paper.
    #[arg(long = "no-yield", default_value_t = false)]
    no_yield: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let config = Config {
        skip: args.skip,
        init_duration: Duration::from_nanos(args.init_ns),
        emit_yield: !args.no_yield,
        big_endian: args.big_endian,
    };

    let mut generator = Generator::new(config);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let result = generator.stream_to(&mut out).and_then(|_| out.flush());

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("s-speedy64: {e}");
            ExitCode::FAILURE
        }
    }
}