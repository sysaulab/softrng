
use clap::{ArgGroup, Parser};
use rand_core::{RngCore, SeedableRng};
use rand_pcg::{Pcg32, Pcg64, Pcg64Mcg};
use std::io::{stdin, stdout, Read, Write};

const BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Selectable PCG variants
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RngVariant {
    Pcg32,
    Pcg64,
    Pcg64Mcg,
}

impl RngVariant {
    /// Number of bytes required for the seed of this RNG
    fn seed_size(&self) -> usize {
        match self {
            RngVariant::Pcg32 => 16,      // [u8; 16]
            RngVariant::Pcg64 => 32,      // [u8; 32]
            RngVariant::Pcg64Mcg => 16,   // [u8; 16]
        }
    }

    /// Build a boxed `RngCore` from a seed buffer of the correct length
    #[allow(deprecated)] // RngCore is deprecated but still object‑safe and usable
    fn build_rng(&self, seed: &[u8]) -> Box<dyn RngCore + Send> {
        match self {
            RngVariant::Pcg32 => {
                let arr: [u8; 16] = seed.try_into().unwrap();
                Box::new(Pcg32::from_seed(arr))
            }
            RngVariant::Pcg64 => {
                let arr: [u8; 32] = seed.try_into().unwrap();
                Box::new(Pcg64::from_seed(arr))
            }
            RngVariant::Pcg64Mcg => {
                let arr: [u8; 16] = seed.try_into().unwrap();
                Box::new(Pcg64Mcg::from_seed(arr))
            }
        }
    }
}

/// Command‑line arguments
#[derive(Parser)]
#[command(about = "Generate an infinite stream of random bytes using a PCG PRNG.",
          group = ArgGroup::new("rng").multiple(false).required(false))]
struct Args {
    /// Select Pcg32 (pcg32, 32‑bit output, 64‑bit state)
    #[arg(long, group = "rng")]
    pcg32: bool,

    /// Select Pcg64 (pcg64, 64‑bit output, 128‑bit state)
    #[arg(long, group = "rng")]
    pcg64: bool,

    /// Select Pcg64Mcg (pcg64_fast, 64‑bit output, 128‑bit state, faster)
    #[arg(long, group = "rng")]
    pcg64_mcg: bool,
}

impl Args {
    /// Determine which RNG variant was selected, defaulting to Pcg64 if none.
    fn variant(&self) -> RngVariant {
        if self.pcg32 {
            RngVariant::Pcg32
        } else if self.pcg64_mcg {
            RngVariant::Pcg64Mcg
        } else {
            // default if no flag is given (or --pcg64 is given)
            RngVariant::Pcg64
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let variant = args.variant();
    let seed_size = variant.seed_size();

    // Read the seed of the exact required size from stdin
    let mut seed = vec![0u8; seed_size];
    stdin().read_exact(&mut seed)?;

    // Instantiate the chosen RNG
    let mut rng = variant.build_rng(&seed);

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut stdout = stdout().lock();

    loop {
        rng.fill_bytes(&mut buffer);
        stdout.write_all(&buffer)?;
    }
}