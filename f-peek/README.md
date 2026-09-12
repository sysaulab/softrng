
# f-peek

A **filter** that passes stdin to stdout while displaying a live hex preview, total bytes transferred, and current throughput on stderr. Designed to give a quick visual check of data flowing through a pipeline without interfering with the data stream.

## Usage

```
f-peek
```

Reads from **stdin**, writes unchanged data to **stdout**, and displays on **stderr**:

- A hex preview of the first 8 bytes of the most recent chunk (or all bytes if less than 8).
- Total bytes processed (in human‑readable binary units: b, Kb, Mb, Gb, etc.).
- Current throughput (bytes/sec, same units).

The display updates at approximately **30 FPS** (configurable constant) to minimize performance impact.

### Example

```bash
# Monitor throughput and preview of an infinite random stream
s-entropy | f-peek > /dev/null

# Preview the first bytes of a file
s-file data.bin | f-peek | f-hex | head -c 32

# Check live speed of a PRNG
f-xoshiro --xoshiro256 < seed.bin | f-peek > /dev/null
```

## Performance

`f-peek` is optimized for minimal overhead:

- Uses a **64 KiB buffer**.
- The display refresh is throttled to ~30 Hz and runs in a separate thread, using a lightweight atomic for time updates. This avoids any system call in the main data path for timing.
- Measured throughput exceeding **13 GB/s** when piping from `/dev/zero` to `/dev/null` (on capable hardware).

The actual throughput depends on the source, sink, and system speed, but `f-peek` itself adds only negligible overhead.

## How It Works

1. Reads data in 64 KiB chunks.
2. Writes each chunk immediately to stdout.
3. Maintains a background thread that updates a shared atomic timestamp every ~33 ms.
4. The main thread checks that timestamp before writing to stderr, so the expensive display operations only occur at the desired frequency (or on the first chunk).

The hex preview shows the first up to 8 bytes of the latest chunk, in lowercase.

## Building

From the workspace root:

```bash
cargo build --release -p f-peek
```

The binary will be at `target/release/f-peek`.

## Part of SoftRNG

`f-peek` is a **filter** (`f-` prefix) in the SoftRNG suite. It does not consume input but passes it through, making it safe to insert anywhere in a pipeline for monitoring.

## License

Distributed under the GNU Affero General Public License version 3 (AGPL-3.0). See the root `LICENSE` file for details.
