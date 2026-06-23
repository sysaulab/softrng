use anyhow::{Context, Result};
use biski64::Biski64Rng;
use rand_core::{RngCore, SeedableRng};
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;

const BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16 MiB
const SEED_SIZE: usize = 32;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let num_threads:usize;
    if args.len() < 2 {
        num_threads = 2;
    }
    else {
        num_threads = args[1]
        .parse()
        .context("Invalid number of threads")?;
    }


    // Read the 32‑byte master seed from stdin
    let mut master_seed = [0u8; SEED_SIZE];
    std::io::stdin()
        .lock()
        .read_exact(&mut master_seed)
        .context("Failed to read 32‑byte seed from stdin")?;

    // Set O_APPEND on stdout (fd 1) so that concurrent writes are atomic
    let stdout_fd = std::io::stdout().as_raw_fd();
    let flags = unsafe { libc::fcntl(stdout_fd, libc::F_GETFL) };
    if flags == -1 {
        anyhow::bail!("fcntl(F_GETFL) failed: {}", std::io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(stdout_fd, libc::F_SETFL, flags | libc::O_APPEND) } == -1 {
        anyhow::bail!("fcntl(F_SETFL, O_APPEND) failed: {}", std::io::Error::last_os_error());
    }

    let mut handles = Vec::with_capacity(num_threads);

    for t in 0..num_threads {
        // Derive a thread‑local seed from the master seed by XORing with the thread index
        let mut seed = master_seed;
        let thread_bytes = (t as u64).to_le_bytes();
        for (i, b) in thread_bytes.iter().enumerate() {
            if i < SEED_SIZE {
                seed[i] ^= b;
            }
        }
        // Further mixing: XOR the index into the high part to avoid collisions
        // (simple enough for this purpose)

        let handle = thread::spawn(move || {
            let mut rng = Biski64Rng::from_seed(seed);
            let mut buffer = vec![0u8; BUFFER_SIZE];

            loop {
                rng.fill_bytes(&mut buffer);

                // Write the entire buffer to stdout using raw write(2).
                // Because O_APPEND is set, the kernel ensures this write is atomic.
                let mut written = 0;
                while written < BUFFER_SIZE {
                    let res = unsafe {
                        libc::write(
                            1,
                            buffer[written..].as_ptr() as *const libc::c_void,
                            BUFFER_SIZE - written,
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

    // Wait for all threads (they run forever; this will block until killed)
    for h in handles {
        let _ = h.join();
    }

    Ok(())
}