use anyhow::Result;
use std::io::{self, Write, Read};
use softrngproject::BUFFER_SIZE;

fn main() -> Result<()> {
    let stdin = io::stdin().lock();
    let mut reader = io::BufReader::with_capacity(BUFFER_SIZE, stdin);
    let mut stdout = io::stdout().lock();

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut hex_buf = vec![0u8; BUFFER_SIZE * 2]; // fixed size, no resize needed

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        // SAFETY: hex_buf has exactly 2*BUFFER_SIZE bytes, so encoding is safe.
        faster_hex::hex_encode(&buffer[..n], &mut hex_buf[..n * 2])
            .expect("hex encode failed");

        // Write the encoded chunk directly to stdout – no BufWriter copy.
        stdout.write_all(&hex_buf[..n * 2])?;
    }
    // No flush needed; lock is dropped at end of scope.
    Ok(())
}