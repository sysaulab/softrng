use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

const PRIME_STEP: u64 = 7776210437768060567;
const TABLE_ENTRIES: usize = 65536;
const NUM_TABLES: usize = 4;
const MAP_SIZE: usize = TABLE_ENTRIES * NUM_TABLES * 8; // 2 MiB

pub struct PQXO64 {
    table: Arc<[u64; TABLE_ENTRIES * NUM_TABLES]>,
    num_threads: usize,
    block_size: usize,
}

impl PQXO64 {
    /// Create a new parallel generator from a 2 MiB seed.
    ///
    /// `num_threads` defaults to `max(1, available_parallelism / 2)`, clamped to `1..=cpus`.
    pub fn new(seed: &[u8], num_threads: Option<usize>) -> Result<Self, &'static str> {
        if seed.len() != MAP_SIZE {
            return Err("seed must be exactly 2 MiB");
        }

        // Convert seed to table in native endianness (like original Qxo64)
        let mut table = Box::new([0u64; TABLE_ENTRIES * NUM_TABLES]);
        unsafe {
            std::ptr::copy_nonoverlapping(
                seed.as_ptr(),
                table.as_mut_ptr() as *mut u8,
                MAP_SIZE,
            );
        }
        let table: Arc<_> = Arc::new(*table);

        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let default = std::cmp::max(1, cpus / 2);
        let n = num_threads
            .unwrap_or(default)
            .clamp(1, cpus);

        Ok(PQXO64 {
            table,
            num_threads: n,
            block_size: 1024 * 1024, // 1 MiB blocks
        })
    }

    /// Override the write block size (default 1 MiB).
    pub fn block_size(mut self, size: usize) -> Self {
        self.block_size = size;
        self
    }

    /// Fill `output_path` (a file or block device) with random data.
    ///
    /// `device_size` is the total number of bytes to write.
    pub fn fill_device(&self, output_path: &str, device_size: u64) -> io::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .read(false)
            .open(output_path)?;
        let fd = file.as_raw_fd();

        let n_threads = self.num_threads;
        let block_size = self.block_size;
        let total_blocks = device_size / block_size as u64;
        let stride = n_threads as u64 * PRIME_STEP;

        // Shared atomic offset in block units
        let offset_counter = Arc::new(AtomicU64::new(0));
        let table = Arc::clone(&self.table);

        let mut handles = Vec::with_capacity(n_threads);
        for t in 0..n_threads {
            let table = Arc::clone(&table);
            let counter = Arc::clone(&offset_counter);
            let fd = fd; // Safe: only used in pwrite, no lseek

            let handle = thread::spawn(move || {
                // Thread‑local counter: starts at t * PRIME_STEP, advances by stride
                let mut ctr: u64 = t as u64 * PRIME_STEP;

                // Buffer for one block
                let mut buf = vec![0u8; block_size];

                loop {
                    // Claim a block index
                    let block_idx = counter.fetch_add(1, Ordering::Relaxed);
                    if block_idx >= total_blocks {
                        break;
                    }
                    let offset = block_idx * block_size as u64;

                    // Fill buffer with random u64 words, advancing ctr by stride each time
                    let mut buf_pos = 0;
                    while buf_pos + 8 <= block_size {
                        let word = read_chunk(&table, ctr);
                        buf[buf_pos..buf_pos + 8].copy_from_slice(&word.to_ne_bytes());
                        ctr = ctr.wrapping_add(stride);
                        buf_pos += 8;
                    }

                    // Write the block at its computed offset
                    let res = unsafe {
                        libc::pwrite(
                            fd,
                            buf.as_ptr() as *const libc::c_void,
                            block_size,
                            offset as libc::off_t,
                        )
                    };
                    if res < 0 {
                        eprintln!("pwrite error: {}", io::Error::last_os_error());
                        break;
                    }
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }
        Ok(())
    }
}

/// Core mixing function – directly accepts the 64‑bit counter.
#[inline]
fn read_chunk(table: &[u64; TABLE_ENTRIES * NUM_TABLES], counter: u64) -> u64 {
    let i0 = (counter & 0xFFFF) as usize;
    let i1 = ((counter >> 16) & 0xFFFF) as usize;
    let i2 = ((counter >> 32) & 0xFFFF) as usize;
    let i3 = ((counter >> 48) & 0xFFFF) as usize;

    table[i0]
        ^ table[TABLE_ENTRIES + i1]
        ^ table[2 * TABLE_ENTRIES + i2]
        ^ table[3 * TABLE_ENTRIES + i3]
}