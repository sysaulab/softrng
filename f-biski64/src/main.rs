use biski64::Biski64Rng;
//use rand::{RngCore, SeedableRng};
use rand_core::{RngCore, SeedableRng};   // <-- SeedableRng is now in scope
use std::io::{stdin, stdout, Read, Write};

const BUFFER_SIZE: usize = 16 * 1024 * 1024;
const SEED_SIZE: usize = 32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut seed_bytes = [0u8; SEED_SIZE];
    stdin().read_exact(&mut seed_bytes)?;

    let mut rng = Biski64Rng::from_seed(seed_bytes);   // now works

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut stdout = stdout().lock();

    loop {
        rng.fill_bytes(&mut buffer);
        stdout.write_all(&buffer)?;
    }
}