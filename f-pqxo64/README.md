
# f-pqxo64

**Highly experimental parallel version of QXO64** – a table‑based PRNG that can generate massive amounts of pseudo‑random data at very high speed. Designed to **saturate modern NVMe drives** by leveraging multiple threads.

> **⚠ Warning**  
> This tool is **not thoroughly tested** and is **not a secure option**. It exists solely for generating large volumes of random‑looking data quickly. The output is **non‑repeatable** across runs due to thread scheduling (the interleaving of thread outputs is nondeterministic). Do **not** use it for cryptographic purposes, reproducibility, or any application requiring predictable or formally validated randomness.

## Overview

`f-pqxo64` is a parallelized version of `f-qxo64`. It reads a **2 MiB seed** from stdin, then spawns a specified number of threads. Each thread:

- Receives its own copy of the seed table (the table is cloned per thread).
- Is assigned a distinct, non‑overlapping range of the 64‑bit counter space.
- Continuously fills 1 MiB buffers with QXO64 output and writes them to stdout using raw `libc::write`.

The tool enables `O_APPEND` on stdout (file descriptor 1) so that concurrent writes from multiple threads are atomic and do not overwrite each other (on POSIX systems supporting `O_APPEND`).

Because each thread writes its blocks as a single `write(2)` call (up to 1 MiB), the kernel will interleave these blocks but not split them. This provides high throughput while keeping the output stream composed of complete blocks.

## Usage

```
f-pqxo64 <num_threads>
```

- `<num_threads>` – number of worker threads to spawn. Each thread produces its own deterministic subsequence of QXO64.
- The program reads **exactly 2 MiB** per thread from stdin as the seed and then outputs an endless stream of binary data to stdout.
- **No command‑line options other than thread count.**

### Example

```bash
# Generate data as fast as possible using 8 threads
cat seed.bin | f-pqxo64 8 > output.bin

# Write directly to an NVMe drive (unbuffered, if filesystem allows)
f-pqxo64 16 < seed.bin | dd of=/dev/nvme0n1 bs=1M status=progress

# Limit output to a specific size (e.g., 100 GiB)
f-pqxo64 4 < seed.bin | f-limit 100G > data.bin
```

## Performance

The tool is optimized for maximum throughput:

- **1 MiB block size** reduces overhead from many small writes.
- **Raw `write(2)` calls** avoid buffering and locking overhead.
- **`O_APPEND`** ensures atomic append without requiring explicit file locking.
- **Multiple threads** allow the generator to scale across CPU cores and better utilize storage bandwidth.

On modern hardware, this can easily produce **tens of gigabytes per second**, limited mainly by CPU, memory bandwidth, and storage write speed.

## How It Works

1. Reads a 2 MiB seed from stdin.
2. Clones the seed for each thread.
3. Splits the 64‑bit counter space evenly:  
   `start_counter = thread_index * (u64::MAX / num_threads)`
4. Each thread creates a `Qxo64` instance from its seed copy and enters an infinite loop:
   - Fills a 1 MiB buffer with QXO64 words (using `read_chunk(counter)` and advancing the counter).
   - Writes the buffer to stdout using `libc::write(1, ...)`.
5. The main thread waits for all threads (they never terminate).

Because the counter ranges are disjoint and QXO64 is deterministic given a seed and counter, each thread's output is deterministic, but the global interleaving of blocks is nondeterministic due to thread scheduling.

## Limitations & Cautions

- **Non‑repeatable output** – the exact byte stream varies between runs because thread scheduling affects the order in which blocks are written. There is no attempt to synchronize or order the output.
- **Not cryptographic** – QXO64 is a simple table‑based PRNG and is not designed to resist adversarial attacks. The parallel version does not improve this; it only increases speed.
- **Unix‑only** – uses `libc::fcntl` and `libc::write`; will not compile on non‑POSIX systems.
- **Untested** – this is an experimental tool, not part of the main tested suite. Use at your own risk.
- **Requires a 2 MiB seed** – the same as `f-qxo64`. If the seed is not exactly 2 MiB, the program errors out.
- **No graceful shutdown** – the threads run forever; kill the process (Ctrl+C) to stop.
- **Memory usage** – each thread allocates a copy of the 2 MiB table plus a 1 MiB buffer. With many threads, memory usage grows linearly.

## Building

From the workspace root:

```bash
cargo build --release -p f-pqxo64
```

The binary will be at `target/release/f-pqxo64`.

## Part of SoftRNG

`f-pqxo64` is a **filter** (`f-` prefix) in the SoftRNG suite. It is the parallel counterpart to `f-qxo64` and is intended for high‑throughput generation scenarios.

## License

Distributed under the GNU Affero General Public License version 3 (AGPL-3.0). See the root `LICENSE` file for details.
