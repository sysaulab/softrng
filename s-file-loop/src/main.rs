use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;
use std::io::Seek;

const BUFFER_SIZE: usize = 64 * 1024;

fn main() {
    // Get the file path from the first argument (no options)
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: s-file-loop <file>");
        process::exit(1);
    }
    let path = &args[1];

    // Open the file once
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("s-file-loop: cannot open '{}': {}", path, e);
            process::exit(1);
        }
    };

    // Reusable buffer
    let mut buf = [0u8; BUFFER_SIZE];
    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Infinite loop: read and write, seek back to start on EOF
    loop {
        match file.read(&mut buf) {
            Ok(0) => {
                // EOF reached, rewind to beginning and continue
                if let Err(e) = file.seek(std::io::SeekFrom::Start(0)) {
                    eprintln!("s-file-loop: seek failed: {}", e);
                    process::exit(1);
                }
            }
            Ok(n) => {
                if let Err(e) = out.write_all(&buf[..n]) {
                    eprintln!("s-file-loop: write failed: {}", e);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("s-file-loop: read error: {}", e);
                process::exit(1);
            }
        }
    }
}
