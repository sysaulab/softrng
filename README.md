# SoftRNG: Random Number Generation and Testing Toolkit

SoftRNG is a modular suite of command-line tools for generating, filtering, and testing random number streams. It is organized as a Cargo workspace containing multiple independent binaries and libraries, each with a specific role. The project emphasizes composability via Unix pipelines: each tool reads from standard input and writes to standard output (or consumes input to produce a summary), allowing flexible construction of RNG pipelines.

The suite includes:

- **Sources** that generate raw random or deterministic data.
- **Filters** that transform or limit data.
- **Terminators/tests** that analyze data quality or consume data.
- **Libraries** for core RNG algorithms.
- **Utility tools** for profiling and manual viewing.

## Building

Requires Rust (stable, 1.70+ recommended). Clone the repository and build all tools with:

```bash
cargo build --release
```

The compiled binaries will be in `target/release/`. Alternatively, build individual tools:

```bash
cargo build --release -p f-peek
```

## Usage

All tools follow Unix conventions: read from stdin, write to stdout, use stderr for diagnostics or progress. Combine them with pipes:

```bash
# Generate 1 MiB of system entropy, convert to hex, save to file
s-entropy | f-limit 1M | f-hex > entropy.hex

# Monitor throughput of the SEEDY64 chaotic generator
s-seedy64 | f-peek > /dev/null

# Test the bit spectrum of a PRNG
f-xoshiro --xoshiro256 < seed.bin | t-bits 24
```

The `softrng` command (when run without arguments) opens an interactive pager with the full manual, including a list of all tools and their options.

```bash
cargo run --release -p softrng
```

## Component Overview

### Sources (`s-*`)

| Tool | Description |
|------|-------------|
| `s-entropy` | High‑quality entropy from the kernel (`getrandom`). Outputs raw binary. |
| `s-file <FILE> [OFFSET]` | Outputs raw content of `FILE`, optionally starting at byte `OFFSET`. |
| `s-file-loop <FILE>` | Outputs file content repeatedly in an endless loop. |
| `s-seedy64` | Continuous chaotic stream from the SEEDY64 mixer (nondeterministic). |
| `s-seedy64-reset` | Outputs one 64‑bit value from a *fresh* SEEDY64 instance each iteration (for testing). |
| `s-zeros` | Infinite stream of zero bytes (as little‑endian 64‑bit words). |

### Filters (`f-*`)

| Tool | Description |
|------|-------------|
| `f-biski64` | Deterministic PRNG (Biski64). Reads a 32‑byte seed from stdin, outputs endless raw bytes. |
| `f-file <FILE>` | Tee stdin to `FILE` while passing through to stdout. |
| `f-hex` | Convert binary stdin to lowercase hexadecimal (two hex digits per byte). |
| `f-limit <SIZE>` | Pass only the first `SIZE` bytes. Supports suffixes: `k`, `m`, `g`, `t` (decimal) or `K`, `M`, `G`, `T` (binary). |
| `f-peek` | Pass stdin to stdout while displaying a hex preview of first 8 bytes, total bytes read, and throughput on stderr. |
| `f-pbiski64` | Parallel version of Biski64; reads a 32‑byte seed, spawns threads to write large blocks atomically. |
| `f-pcg` | PCG PRNG family (`--pcg32`, `--pcg64`, `--pcg64_mcg`). Reads seed of appropriate size (16 or 32 bytes). |
| `f-pqxo64` | Parallel version of QXO64 (2 MiB table‑based PRNG); reads a 2 MiB seed, spawns threads for high throughput. |
| `f-qxo64` | Table‑based PRNG (QXO64). Reads a 2 MiB seed from stdin, outputs endless 64‑bit words. |
| `f-reduce <0|1|2|3>` | Reduce bit width of a 64‑bit stream: 0 (pass), 1 (32‑bit), 2 (16‑bit), 3 (8‑bit). |
| `f-xoshiro` | Xoshiro/xoroshiro family (multiple variants via flags). Reads seed of size depending on variant (16, 32, or 64 bytes). |

### Terminators / Tests (`t-*`)

| Tool | Description |
|------|-------------|
| `t-bits [BITS]` | Bit‑spectrum test: reads n‑bit words (default 24) and counts unique values, giving a statistical score. |
| `t-hole` | Discard all input (like `/dev/null`). |
| `t-nist-quick` | Runs a subset of NIST SP 800‑22 tests on 1 MiB blocks (Frequency, Block Frequency, Runs, Longest Run, Cumulative Sums). |
| `t-nist-all` | Runs the full NIST SP 800‑22 suite on blocks (requires minimum 125 KB per block). |

### Libraries

| Crate | Description |
|-------|-------------|
| `qxo64` | Deterministic PRNG based on a 2 MiB seed table (see [`qxo64/README.md`](qxo64/README.md)). |
| `seedy64` | Nondeterministic chaotic entropy source exploiting data races (see [`seedy64/PAPER.md`](seedy64/PAPER.md)). |

### Utility Tools

| Tool | Description |
|------|-------------|
| `softrng` | Interactive viewer for the full manual (`assets/manual.txt`). |
| `timer-profile` | Profiles OS sleep timer granularity by measuring a decaying sequence of delays (see [`timer-profile/PAPER.md`](timer-profile/PAPER.md)). |

## Design and Performance

The tools are optimized for high throughput and low overhead. Key design choices:

- Large I/O buffers (64 KiB typical, 16 MiB for parallel filters).
- Direct use of raw `write(2)` with `O_APPEND` in parallel filters for atomic writes.
- Avoidance of unnecessary copying; e.g., `bytemuck::cast_slice` for u64-to-bytes conversion.
- Throttled progress displays to minimize performance impact.

The suite includes fast deterministic PRNGs (Biski64, QXO64, Xoshiro, PCG) and an experimental nondeterministic source (`seedy64`) that exploits hardware nondeterminism via data races.

## Testing Your Own Data

You can test any binary stream by piping it into `t-bits` or `t-nist-*`. For example:

```bash
# Test the first 100 MB of a file with the bit spectrum test at 24 bits
s-file mydata.bin | f-limit 100M | t-bits 24

# Run NIST quick tests on a PRNG stream
f-xoshiro --xoshiro256 < seed.bin | f-limit 1G | t-nist-quick
```

## Platform Support

Most tools are cross-platform. The parallel filters (`f-pqxo64`, `f-pbiski64`) and `s-file-loop` rely on Unix-specific features (`fcntl`, `O_APPEND`, `libc`). On non-Unix systems these may not compile. The rest should work on Linux, macOS, and Windows (with possible minor adjustments).

## Documentation

- **Full manual**: run `softrng` or view [`softrng/assets/manual.txt`](softrng/assets/manual.txt).
- **QXO64**: see [`qxo64/README.md`](qxo64/README.md).
- **SEEDY64**: see [`seedy64/PAPER.md`](seedy64/PAPER.md).
- **Timer profiling**: see [`timer-profile/PAPER.md`](timer-profile/PAPER.md).
- **Bit spectrum test**: see [`t-bits/README.md`](t-bits/README.md).

## License

The workspace is licensed under the GNU Affero General Public License version 3 (AGPL-3.0) for the `softrng` tool and overall suite. Individual crates may have different licenses; check each crate's `Cargo.toml` and source files for details.