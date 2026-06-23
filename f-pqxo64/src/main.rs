use anyhow::{Context, Result};
use qxo64::{Qxo64, MAP_SIZE};
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;

const BLOCK_SIZE: usize = 1024 * 1024; // 1 MiB

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <num_threads>", args[0]);
        std::process::exit(1);
    }

    let num_threads: usize = args[1]
        .parse()
        .context("Invalid thread count")?;

    // Read the 2 MiB seed from stdin
    let mut seed_bytes = vec![0u8; MAP_SIZE];
    std::io::stdin()
        .lock()
        .read_exact(&mut seed_bytes)
        .context("Failed to read 2 MiB seed from stdin")?;

    // Enable O_APPEND on stdout so that concurrent writes are atomic
    // and do not overwrite each other.
    let stdout_fd = std::io::stdout().as_raw_fd();
    let flags = unsafe { libc::fcntl(stdout_fd, libc::F_GETFL) };
    if flags == -1 {
        anyhow::bail!("fcntl(F_GETFL) failed: {}", std::io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(stdout_fd, libc::F_SETFL, flags | libc::O_APPEND) } == -1 {
        anyhow::bail!("fcntl(F_SETFL, O_APPEND) failed: {}", std::io::Error::last_os_error());
    }

    // Each thread will get a unique starting counter.
    // We split the 64‑bit space evenly among threads.
    let counter_step = u64::MAX / num_threads as u64;

    let mut handles = Vec::with_capacity(num_threads);

    for t in 0..num_threads {
        let seed = seed_bytes.clone(); // each thread owns its own table
        let start_counter = t as u64 * counter_step;

        let handle = thread::spawn(move || {
            let qxo = Qxo64::new_from_data(&seed).expect("Failed to create Qxo64");
            let mut counter = start_counter;
            let mut buf = vec![0u8; BLOCK_SIZE];

            loop {
                // Fill the buffer with random words
                let mut pos = 0;
                while pos + 8 <= BLOCK_SIZE {
                    let word = qxo.read_chunk(counter);
                    buf[pos..pos + 8].copy_from_slice(&word.to_ne_bytes());
                    counter = counter.wrapping_add(1);
                    pos += 8;
                }

                // Write the block to stdout (fd 1) – O_APPEND ensures atomic append
                let mut written = 0;
                while written < BLOCK_SIZE {
                    let res = unsafe {
                        libc::write(
                            1,
                            buf[written..].as_ptr() as *const libc::c_void,
                            BLOCK_SIZE - written,
                        )
                    };
                    if res < 0 {
                        eprintln!("write error: {}", std::io::Error::last_os_error());
                        return;
                    }
                    written += res as usize;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads (they run forever, so this will block until killed)
    for h in handles {
        h.join().unwrap();
    }

    Ok(())
}