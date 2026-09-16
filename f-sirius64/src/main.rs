//Ported from C: https://github.com/matteo65/Sirius64

use std::io::{self, Read, Write};

#[inline(always)]
fn sirius64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = 0x9e37_79b9_7f4a_7c15u64.wrapping_mul(z ^ (z >> 17));
    z = z.rotate_left(32);
    0x9e37_79b9_7f4a_7c15u64.wrapping_mul(*state ^ z ^ (z >> 17))
}

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut seed = [0u8; 8];
    stdin.read_exact(&mut seed)?;
    let mut state = u64::from_le_bytes(seed);

    let mut buf = [0u8; 8 * 4096];
    loop {
        for chunk in buf.chunks_exact_mut(8) {
            let x = sirius64(&mut state);
            chunk.copy_from_slice(&x.to_le_bytes());
        }
        stdout.write_all(&buf)?;
    }
}