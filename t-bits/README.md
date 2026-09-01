
# t-bits

A high-performance bit spectrum tester for analyzing the distribution of low/high halves of 64-bit words in binary data streams. Designed to evaluate the quality of random number generators and detect biases in binary data.

**Default mode**: 24-bit space, stage 0 only — a quick sanity check that completes in seconds to minutes and provides a strong statistical profile of any generator.

## Overview

This tool processes 64-bit words from standard input, extracting both the low and high 32-bit halves, and tracks unique values using a bitset. It calculates a statistical score based on the expected number of unique values (birthday problem/occupancy distribution) to determine if the input data exhibits uniform random distribution.

The tester progresses through stages (0–10), where each stage represents increasingly stringent coverage requirements of the value space, from initial coverage (stage 0) to complete saturation of all possible values (stage 10).

**Key insight**: Each stage requires approximately **2.3× more data** than the previous one—a natural emergent property of the birthday problem mathematics. This means stage 0 provides the best “information per byte” ratio, making it ideal for quick profiling.

## Features

- **High performance**: Processes data in 64 KiB chunks with direct memory mapping to u64 slices
- **Memory efficient**: Uses a single bitset to track unique values (1 bit per possible value)
- **Statistical scoring**: Compares actual unique count against theoretical expectation from uniform random distribution
- **Multi-stage testing**: Progresses through 11 stages (0–10) with configurable stopping point
- **Progress monitoring**: Throttled updates (~10 FPS) to minimize performance impact
- **CSV logging**: Optional detailed progress logging for analysis
- **Flexible bit widths**: Test 1–34 bit spaces (34-bit requires ~2 GiB RAM)
- **Sane defaults**: 24-bit space, stage 0 only — quick sanity check out of the box

## Installation

### Prerequisites
- Rust toolchain (1.70+ recommended)
- 64-bit system (required for optimal performance)

### Build
```bash
cargo build --release
```

The optimized binary will be at `target/release/t-bits`.

## Usage

### Basic usage (default: quick sanity check)
```bash
# Runs stage 0 at 24 bits (space = 16.7M values) — completes in seconds to minutes
./t-bits < random_data.bin

# Equivalent explicit command
./t-bits 24 -s 0 < random_data.bin
```

### Command line options
```
Usage: t-bits [OPTIONS] [BITS]

Arguments:
  [BITS]  Bit width to test (1–34). 33/34 may require 1–2 GiB of RAM [default: 24]

Options:
  -s, --stage-max <STAGE_MAX>        Maximum stage (0–10). Stage 0 completes after 1× space coverage [default: 0]
  -l, --log <FILE>                   Enable logging to CSV file (optional filename, default: bspec.log)
  -r, --results <RESULTS>            Results file (final stage scores) [default: bspec32.txt]
  -q, --quiet                        Quiet mode: suppress stderr progress updates
  -u, --update-interval <UPDATE_INTERVAL>  Minimum seconds between progress updates [default: 0.1]
  -h, --help                         Print help
  -V, --version                      Print version
```

### Quick Start Guide

**The 23→32 sweet spot**: For meaningful results, use bit widths between 23 and 32. The default 24-bit stage‑0 test is the recommended starting point.

| Bits | Space | Samples for 63.2% | Time (fast RNG) | Use Case |
|------|-------|-------------------|-----------------|----------|
| 20 | 1M | 1M | ~1 sec | Smoke test |
| 23 | 8M | 8M | ~10 sec | Quick validation |
| **24** | **16.7M** | **16.7M** | **~30 sec** | **Default sanity check** |
| 26 | 67M | 67M | ~1 min | Standard test |
| 29 | 537M | 537M | ~10 min | Thorough |
| 32 | 4.3B | 4.3B | ~1 hr | Deep analysis |

**Why 23 bits minimum**: Below 23 bits, the space is too small to get statistically significant results—you hit full coverage too quickly to distinguish good from bad generators.

**Why 32 bits maximum**: Above 32 bits, memory requirements (2+ GiB) and processing time become prohibitive for most use cases.

### Examples

**Quick sanity check (default)**:
```bash
./t-bits < generator_output.bin
```
Completes with 16.7M samples; score near 1.0 indicates good randomness.

**Explicit quick test**:
```bash
./t-bits 20 -s 0 < generator_output.bin
```

**Standard validation (~1 minute)**:
```bash
./t-bits 26 -s 0 < generator_output.bin
```

**Thorough analysis (~10 minutes)**:
```bash
./t-bits 29 -s 5 < generator_output.bin
```

**Deep dive (~1 hour)**:
```bash
./t-bits 32 -s 10 < generator_output.bin
```

## Understanding the Score

The score represents the ratio of **actual unique values** to **expected unique values**:

```
expected_unique = space * (1 - e^(-n/space))
score = actual_unique / expected_unique
```

Where:
- `space` = 2^bits (total possible values)
- `n` = number of samples processed

### Score interpretation:
- **~1.0**: Data matches uniform random distribution (ideal)
- **< 1.0**: More repeats than expected (potential bias or correlation)
- **> 1.0**: Impossible in theory, indicates measurement error

### Stage 0: The Sweet Spot

At stage 0 (1× space coverage), you achieve **63.2% unique coverage**—this is the mathematically optimal point for profiling:

- **Maximum information per byte**: Stage 0 gives you the strongest statistical signal per unit of data processed
- **Sufficient for most purposes**: Detects virtually all biases, correlations, and patterns that matter in practice
- **Each subsequent stage requires 2.3× more data**: For marginal additional information
- **Most real-world applications** don't need to distinguish between “very good” and “perfectly uniform”

The 2.3× progression between stages emerges naturally from the birthday problem mathematics—it wasn't designed, it's a fundamental property of random sampling.

### Stages:
- **Stage 0**: Completes after processing `space` samples (1× coverage, 63.2% unique expected)
- **Stages 1–9**: Progress as score exceeds thresholds (0.9, 0.99, 0.999, …)
- **Stage 10**: Achieved when all possible values observed

### When higher stages matter:
- Cryptography research
- Extreme-quality requirements (lottery, gambling)
- Detecting state compromise in CSPRNGs after billions of outputs
- Academic validation of new RNG algorithms

## Output

### Progress display (stderr)
```
progress: 45.23%  score: 0.999834721  stage: 0/0
```

### Results file
```
stage 0 : 0.999998234
```

### CSV log format
```csv
progress,score,seconds
0.001234,0.998765,0.123
0.002468,0.999123,0.245
```

## Performance

- **Processing speed**: fast
- **Memory usage**: `2^(bits-3)` bytes (e.g., 2 MiB for 24-bit space)
- **Optimization**: Throttled progress updates prevent I/O bottleneck
- **Scaling**: Each stage requires ~2.3× more data than the previous (emergent property)

## How It Works

1. Reads binary data in 64 KiB chunks
2. Reinterprets bytes as u64 words (little-endian)
3. For each word, extracts low and high 32-bit halves
4. Masks to the target bit width
5. Marks values in a bitset, tracking first occurrences
6. Calculates statistical score against theoretical expectation
7. Progresses through stages as coverage improves

## Limitations

- **Input size**: Stage 10 may require processing petabytes of data for 32-bit space
- **Memory**: Higher bit widths require significant RAM (34-bit = 2 GiB)
- **Endianness**: Currently assumes little-endian input (x86/ARM)
- **Statistical floor**: Bit widths below 23 provide limited statistical power

## Contributing

Feel free to open issues or submit pull requests for:
- Performance improvements
- Additional statistical tests
- Support for big-endian systems
- Memory-mapped file input
- Multi-threading

## Author

Created as a tool for testing random number generator quality and detecting biases in binary data streams.

## Acknowledgments

Special thanks to the emergent properties of the birthday problem that make this tool both elegant and practical.
