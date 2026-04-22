use anyhow::{bail, Context, Result};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use softrng::BUFFER_SIZE;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("sr_rngoutputshim: missing RNG_output argument");
        std::process::exit(1);
    }
    let rng_arg = &args[1];

    // Read 8-byte seed from stdin
    let mut seed_bytes = [0u8; 8];
    io::stdin()
        .read_exact(&mut seed_bytes)
        .context("sr_rngoutputshim: error reading stdin")?;
    let seed = u64::from_le_bytes(seed_bytes);

    // Build command: RNG_output <arg> inf 0123456789abcdef
    let cmd_str = format!("RNG_output {} inf {:016x}", rng_arg, seed);
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&cmd_str)
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("sr_rngoutputshim: cannot open {}", cmd_str))?;

    let mut child_stdout = child.stdout.take().expect("failed to capture stdout");
    let mut stdout = io::stdout().lock();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        // Read exactly BUFFER_SIZE bytes (as C code expects)
        match child_stdout.read_exact(&mut buffer) {
            Ok(()) => {
                // Write exactly BUFFER_SIZE bytes
                if let Err(e) = stdout.write_all(&buffer) {
                    eprintln!(
                        "sr_rngoutputshim: error writing stdout. ({},{})",
                        BUFFER_SIZE, BUFFER_SIZE
                    );
                    bail!(e);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Partial read: C code treats this as error
                eprintln!(
                    "sr_rngoutputshim: error reading child. ({},{})",
                    buffer.len(), BUFFER_SIZE
                );
                break;
            }
            Err(e) => {
                eprintln!(
                    "sr_rngoutputshim: error reading child. ({},{})",
                    0, BUFFER_SIZE
                );
                bail!(e);
            }
        }
    }

    let _ = child.wait();
    Ok(())
}