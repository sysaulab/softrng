use anyhow::{Result};
use clap::Parser;
use crossbeam_channel::{bounded};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;

#[derive(Parser)]
#[command(name = "f-multi", about = "Multiplex stdin to multiple shell softrngs")]
struct Args {
    /// Shell commands to execute. Each command can include redirections (e.g., "grep foo > out.txt").
    commands: Vec<String>,
    /// Buffer size per child (in bytes). Increase if one consumer is slow.
    #[arg(short, long, default_value = "65536")]
    buffer: usize,
    /// Stop all processes if any child exits early.
    #[arg(short, long)]
    stop_on_error: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.commands.is_empty() {
        anyhow::bail!("At least one command required");
    }

    let main_tx = bounded::<Vec<u8>>(args.commands.len());
    let mut handles = Vec::new();
    let mut child_txs = Vec::new();

    // Spawn one thread per command
    for (_i, cmd_str) in args.commands.iter().enumerate() {
        let (tx, rx) = bounded::<Vec<u8>>(1); // one chunk buffer per child
        child_txs.push(tx);
//        let main_rx_clone = main_rx.clone();
        let cmd = cmd_str.clone();
//        let buffer_size = args.buffer;
        let stop_on_error = args.stop_on_error;

        let handle = thread::spawn(move || {
            // Spawn the shell command
            let mut child = match Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[f-multi] Failed to spawn '{}': {}", cmd, e);
                    if stop_on_error {
                        std::process::exit(1);
                    }
                    return;
                }
            };

            let mut child_stdin = child.stdin.take().expect("failed to get stdin");

            // Forward data from main_rx to child's stdin
            while let Ok(data) = rx.recv() {
                if let Err(e) = child_stdin.write_all(&data) {
                    eprintln!("[f-multi] Write to '{}' failed: {}", cmd, e);
                    break;
                }
                // Flush to avoid deadlocks
                let _ = child_stdin.flush();
            }

            // Close stdin and wait for child
            drop(child_stdin);
            let status = child.wait().unwrap_or_else(|e| {
                eprintln!("[f-multi] Wait error for '{}': {}", cmd, e);
                std::process::ExitStatus::default()
            });
            if !status.success() && stop_on_error {
                eprintln!("[f-multi] Command '{}' exited with {}", cmd, status);
                std::process::exit(1);
            }
        });
        handles.push(handle);
    }

    // Main thread: read stdin and broadcast
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buffer = vec![0u8; args.buffer];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        let chunk = buffer[..n].to_vec();
        for tx in &child_txs {
            if tx.send(chunk.clone()).is_err() {
                eprintln!("[f-multi] A child closed its channel, stopping broadcast.");
                if args.stop_on_error {
                    std::process::exit(1);
                }
                break;
            }
        }
    }

    // Close all sending channels
    drop(child_txs);
    drop(main_tx);

    // Wait for all writer threads
    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}
