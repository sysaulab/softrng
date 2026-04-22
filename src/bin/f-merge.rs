use anyhow::{bail, Context, Result};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use softrng::BUFLEN;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("f-merge: must supply arguments");
        std::process::exit(1);
    }

    let command = args.join(" ");
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("f-merge: cannot execute '{}'", command))?;

    let mut right_stream = child.stdout.take().expect("failed to capture stdout");
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut left_buf = vec![0u64; BUFLEN];
    let mut right_buf = vec![0u64; BUFLEN];

    loop {
        // Read u64 chunks from stdin and command output
        let left_read = read_u64_chunk(&mut stdin, &mut left_buf)?;
        let right_read = read_u64_chunk(&mut right_stream, &mut right_buf)?;

        if left_read != right_read {
            bail!("f-merge: read mismatch (stdin: {}, command: {})", left_read, right_read);
        }
        if left_read == 0 {
            break;
        }

        // XOR the buffers
        for i in 0..left_read {
            left_buf[i] ^= right_buf[i];
        }

        // Write result
        let bytes = unsafe {
            std::slice::from_raw_parts(
                left_buf.as_ptr() as *const u8,
                left_read * std::mem::size_of::<u64>(),
            )
        };
        stdout.write_all(bytes)?;
    }

    child.wait()?;
    Ok(())
}

/// Read as many u64 elements as possible (up to buf.len()) and return count.
fn read_u64_chunk<R: Read>(reader: &mut R, buf: &mut [u64]) -> io::Result<usize> {
    let bytes = unsafe {
        std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * 8)
    };
    let n = reader.read(bytes)?;
    Ok(n / 8)
}
