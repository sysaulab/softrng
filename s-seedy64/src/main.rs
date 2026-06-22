use anyhow::Result;
use softrngproject::seedy64::Seedy64;
use std::io::{self, Write};
use softrngproject::BUFFER_SIZE;

fn main() -> Result<()> {
    let mut seedy = Seedy64::new();
    let mut stdout = io::stdout().lock();
    let mut buf = vec![0u8; BUFFER_SIZE];

    loop {
        seedy.fill_buffer(&mut buf);
        stdout.write_all(&buf)?;
        stdout.flush()?;
    }
}