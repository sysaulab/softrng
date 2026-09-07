# qxo64

`qxo64` is a fast, deterministic pseudo-random number generator (PRNG) built around a 2 MiB seed table. It produces high-quality 64-bit values suitable for simulations, procedural generation, and other non-cryptographic use cases.

## Features

- **Deterministic** – Given the same seed, the output sequence is always identical.
- **Fast** – Uses simple arithmetic and XOR operations on a precomputed table.
- **CPU‑friendly** – Leaves the most important CPU ports free for real work by relying only on simple ALU (XOR, shift, multiply) and memory operations.
- **Flexible output** – Generate `u64` words or arbitrary byte buffers at any offset.
- **Simple API** – Load a seed once, then generate unlimited data.
- **Zero dependencies** beyond `libc` and `anyhow` for error handling.

## Installation

Add `qxo64` to your `Cargo.toml`:

```toml
[dependencies]
qxo64 = "0.1.0"
```

## Usage

### Loading a seed

The generator requires a seed of exactly **2 MiB** (`MAP_SIZE` = 65536 × 4 × 8 bytes). You can load it from a file or a raw byte slice:

```rust
use qxo64::Qxo64;
use std::path::Path;

// From a file
let generator = Qxo64::new_from_file(Path::new("seed.bin"))?;

// From a byte slice (must be 2 MiB long)
let seed_bytes: Vec<u8> = std::fs::read("seed.bin")?;
let generator = Qxo64::new_from_data(&seed_bytes)?;
```

### Generating `u64` values

```rust
let generator = Qxo64::new_from_file(Path::new("seed.bin"))?;

// Generate a single u64
let value = generator.read_chunk(0);

// Fill a buffer with consecutive u64s
let mut buffer = vec![0u64; 100];
generator.fill_u64s(&mut buffer, 42); // start at index 42
```

### Generating bytes

You can generate bytes at any byte offset (not necessarily aligned to `u64`):

```rust
let mut buffer = vec![0u8; 1024];
generator.fill_bytes(&mut buffer, 12345); // starts at byte offset 12345
```

## API Overview

### Constants

| Constant        | Value                          | Description                           |
|-----------------|--------------------------------|---------------------------------------|
| `TABLE_ENTRIES` | 65536                          | Number of entries per table           |
| `NUM_TABLES`    | 4                              | Number of tables                      |
| `MAP_SIZE`      | `TABLE_ENTRIES * NUM_TABLES * 8` | Total seed size (2 MiB)               |

### `Qxo64` Methods

| Method                 | Description                                                       |
|------------------------|-------------------------------------------------------------------|
| `new_from_data(bytes)` | Creates a generator from a 2 MiB byte slice.                      |
| `new_from_file(path)`  | Reads a seed file and creates a generator.                        |
| `read_chunk(idx)`      | Returns the `u64` value for index `idx`.                          |
| `fill_u64s(buf, start)`| Fills a `&mut [u64]` with consecutive chunks starting at `start`. |
| `fill_bytes(buf, off)` | Fills a `&mut [u8]` starting at absolute byte offset `off`.       |

## Seed Data

The seed must be exactly **2 MiB** (2,097,152 bytes). Internally, it is interpreted as an array of 262,144 little‑endian `u64` values, split into four tables of 65,536 entries each. The endianness of the seed does not affect the output sequence because the values are only used in XOR operations and the output uses the platform’s native endianness.

### Obtaining a seed

You can generate a seed yourself from any source of entropy (e.g., `/dev/urandom`, a cryptographic PRNG) or use a pre‑existing file. The quality of the seed directly influences the quality of the generated sequence – a poor seed (e.g., all zeros) will produce weak output, though the XOR‑based mixing still provides some scrambling.

## How It Works

For each index `idx`, the generator computes:

```
counter = idx × 0x6BFBA476E2E0D737   (PRIME_STEP)
```

Then it extracts four 16‑bit parts of `counter` and XORs one value from each of the four tables:

```
result = table0[ (counter        ) & 0xFFFF ]
       ^ table1[ (counter >> 16  ) & 0xFFFF ]
       ^ table2[ (counter >> 32  ) & 0xFFFF ]
       ^ table3[ (counter >> 48  ) & 0xFFFF ]
```

This construction is inspired by counter‑based PRNGs like `SipHash` or `AES‑CTR`, where the seed table acts as a substitution box. The large prime step ensures that consecutive indices map to very different table positions, producing good avalanche behaviour.

## Statistical Quality

`qxo64` passes most standard statistical tests (e.g., Dieharder, TestU01 SmallCrush, most of NIST STS). However, it exhibits a detectable bias in the **Longest Run of Ones** test from the NIST suite. This bias is a known consequence of the XOR‑based table construction and the specific prime step. For applications that require passing the full NIST suite or equivalent rigorous testing, consider using a different generator or applying a post‑processing step.

## Performance and Design Goals

`qxo64` is designed to be *fast enough* without becoming the bottleneck when testing much slower PRNGs. It achieves this by leaving the most important CPU ports (e.g., those used for heavy arithmetic, vector units, or branch prediction) free for the application’s real work. The generator itself only uses simple ALU operations (XOR, shift, one multiply) and memory lookups.

**Speed comparison**  
- Slower than `biski64` (a specialised high‑throughput generator).  
- Faster than most other general‑purpose PRNGs.  

This positioning makes `qxo64` ideal for scenarios where you need to generate test vectors quickly but the generator’s own speed should not mask performance issues in the code under test.

### Benchmark

Generating a `u64` requires:
- One 64‑bit multiplication (`wrapping_mul`)
- Four bit‑shifts and masks
- Four table lookups
- Three XOR operations

No branches or bounds checks occur in the hot path (the table access is safe because indices are always < 65536). On modern hardware, it can produce several GB/s of output.

## Safety

The `new_from_data` method uses an `unsafe` block to copy bytes into the `Vec<u64>` table without first converting through a temporary byte buffer. This is safe because:
- The destination is a valid, aligned `Vec<u64>` with exactly the required capacity.
- The source length is checked to be exactly `MAP_SIZE`.
- The byte slice view is created over properly initialized memory (the `Vec<u64>` is zero‑initialized before copying).

All other methods are safe.

## Limitations

- **Not cryptographic**: Do not use for security‑sensitive applications. The generator is not designed to resist adversarial attacks.
- **Seed size**: The seed must be exactly 2 MiB. If you need a smaller footprint, consider hashing a shorter seed to derive the table.
- **Thread safety**: `Qxo64` is not internally synchronised. However, it is read‑only after construction, so it can be shared safely across threads (e.g., with `Arc`).

## License

This crate is distributed under the terms of the MIT license (or Apache 2.0, at your option). See `LICENSE` for details.