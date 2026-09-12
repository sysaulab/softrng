# s-file-loop

A simple **source** that reads a file and outputs its contents **endlessly** to stdout. When EOF is reached, it seeks back to the beginning and continues reading, creating an infinite loop. Useful for turning a finite seed file or test vector into a continuous data stream for pipelines.

## Usage

```
s-file-loop <file>
```

- `<file>` – path to the input file.  
- No options; the file name is the only argument.  
- The program runs **forever** until killed (e.g., Ctrl+C or `kill`).

### Examples

```bash
# Output a 2 MiB seed file repeatedly
s-file-loop seed.bin | f-qxo64 > /dev/null

# Use a short text file as an endless input to f-hex
s-file-loop short.txt | f-hex

# Test the bit spectrum of a repeating 1 MiB pattern
s-file-loop pattern.bin | f-limit 10M | t-bits 24
```

## Behavior

- Reads the file in 64 KiB chunks and writes each chunk to stdout.  
- On EOF, rewinds to byte 0 and continues.  
- If the file cannot be opened or a read/write error occurs, an error message is printed to stderr and the program exits with code 1.  
- If the file is empty, it will simply spin in a tight loop (read returns 0, seek, repeat) without producing output – this is generally not useful.

## Building

From the workspace root:

```bash
cargo build --release -p s-file-loop
```

The binary will be at `target/release/s-file-loop`.

## Part of SoftRNG

`s-file-loop` is a component of the **SoftRNG** suite for generating, filtering, and testing random number streams. It acts as a **source** (`s-` prefix) and requires no input from stdin.

## License

Distributed under the GNU Affero General Public License version 3 (AGPL-3.0). See the root `LICENSE` file for details.
