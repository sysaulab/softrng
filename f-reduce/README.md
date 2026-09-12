```markdown
# f-reduce

A filter that **reduces the bit width** of a 64‑bit binary stream by XOR‑folding the high and low halves. It is designed for **testing and experimentation**, not for improving randomness.

> **⚠ Warning**  
> Using this tool **tends to induce, not remove, statistical anomalies**. The XOR‑folding operation is destructive: it can introduce biases, destroy uniformity, and cause failures in many randomness test suites (e.g., NIST SP 800‑22, Dieharder, TestU01). Do **not** use it to “clean up” a weak generator; it will likely make things worse. Its purpose is to allow controlled degradation of a data stream for analysis.

## Usage

```
f-reduce <0|1|2|3>
```

- `0` – pass‑through (no reduction, outputs the original 64‑bit words unchanged).  
- `1` – reduces to 32‑bit output by XOR‑ing the high and low 32‑bit halves of each 64‑bit word.  
- `2` – reduces to 16‑bit output by first folding to 32 bits, then XOR‑ing the two 16‑bit halves.  
- `3` – reduces to 8‑bit output by folding to 16 bits, then XOR‑ing the two bytes.

Input is read from **stdin** as a stream of 64‑bit little‑endian words. Output is written to **stdout** as the corresponding reduced‑width little‑endian values (32‑bit, 16‑bit, or 8‑bit). Any leftover bytes (< 8) at the end of input are ignored (discarded).

## Examples

```bash
# Reduce a 64‑bit stream to 32‑bit values and save to a file
s-file seed.bin | f-reduce 1 > reduced32.bin

# Reduce to 8‑bit and run a quick bit‑spectrum test
s-file seed.bin | f-reduce 3 | t-bits 8

# Show the effect on a known PRNG:
f-xoshiro --xoshiro256 < seed.bin | f-reduce 2 | f-limit 1M | t-bits 16
```

## How It Works

For each 64‑bit word read from stdin:

1. If quality = 1:  
   `output = (word >> 32) ^ (word & 0xFFFFFFFF)`
2. If quality = 2:  
   `mixed32 = (word >> 32) ^ (word & 0xFFFFFFFF)`  
   `output = (mixed32 >> 16) ^ (mixed32 & 0xFFFF)`
3. If quality = 3:  
   `mixed32 = (word >> 32) ^ (word & 0xFFFFFFFF)`  
   `mixed16 = (mixed32 >> 16) ^ (mixed32 & 0xFFFF)`  
   `output = (mixed16 >> 8) ^ (mixed16 & 0xFF)`

The result is written as a little‑endian integer of the corresponding size.

## Statistical Impact

XOR‑folding can **destroy** the carefully designed statistical properties of a high‑quality generator. Even if the original stream passes tests like Dieharder or NIST, the reduced stream may show:

- Biased bit distribution (e.g., many more 1s than 0s).  
- Correlations between adjacent output values.  
- Failures in frequency, runs, and spectral tests.  
- Loss of entropy per bit (if the original 64‑bit values were independent, the folded output is still random *if* the original was truly random, but real PRNGs have structure that may become apparent after folding).

**Do not assume that reducing bit width preserves randomness.** The tool is intentionally simple and transparent; it is meant to help you explore how transformations affect test results.

## Building

From the workspace root:

```bash
cargo build --release -p f-reduce
```

The binary will be at `target/release/f-reduce`.

## License

This tool is part of the SoftRNG suite and is distributed under the GNU Affero General Public License version 3 (AGPL‑3.0). See the root `LICENSE` file for details.
```