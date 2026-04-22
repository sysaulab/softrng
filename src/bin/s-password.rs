use anyhow::{Result};
use std::env;
use std::io::{self, Write};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let password = if args.len() >= 2 {
        args[1].as_bytes()
    } else {
        let zero = 0u64;
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let bytes = zero.to_ne_bytes();
        loop {
            if handle.write_all(&bytes).is_err() {
                eprintln!("s-password: write error.");
                std::process::exit(1);
            }
        }
    };

    let mut stdout = io::stdout().lock();
    let mut hash = 0u64;

    loop {
        for &byte in password {
            hash = (hash << 7) | (hash >> (64 - 7));
            hash ^= byte as u64;
        }
        if stdout.write_all(&hash.to_ne_bytes()).is_err() {
            eprintln!("s-password: write error.");
            std::process::exit(1);
        }
        stdout.flush()?;
    }
}
