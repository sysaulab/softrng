use anyhow::Result;
use std::io::{self, Write, Read};
use softrng::{HEX_CHARS, BUFFER_SIZE};

fn main() -> Result<()> {
    let stdin = io::stdin().lock();
    let mut reader = io::BufReader::with_capacity(BUFFER_SIZE, stdin);
    let stdout = io::stdout().lock();
    let mut writer = io::BufWriter::with_capacity(BUFFER_SIZE * 2, stdout);

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut hex_buf = Vec::with_capacity(BUFFER_SIZE * 2);

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        hex_buf.clear();
        for &byte in &buffer[..n] {
            hex_buf.push(HEX_CHARS[(byte >> 4) as usize]);
            hex_buf.push(HEX_CHARS[(byte & 0x0f) as usize]);
        }
        writer.write_all(&hex_buf)?;
    }
    writer.flush()?;
    Ok(())
}
