use clap::{ArgGroup, Parser};
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::{
    Seed512,                                       // for the 512‑bit generators
    Xoroshiro128Plus, Xoroshiro128StarStar,
    Xoshiro128Plus, Xoshiro128StarStar,
    Xoshiro256Plus, Xoshiro256StarStar,
    Xoshiro512Plus, Xoshiro512StarStar,
};
use std::io::{stdin, stdout, Read, Write};

const BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Selectable PRNG variants
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RngVariant {
    Xoroshiro128,
    Xoroshiro128Plus,
    Xoshiro128,
    Xoshiro128Plus,
    Xoshiro256,
    Xoshiro256Plus,
    Xoshiro512,
    Xoshiro512Plus,
}

impl RngVariant {
    /// Number of bytes required for the seed of this RNG
    fn seed_size(&self) -> usize {
        match self {
            RngVariant::Xoroshiro128 | RngVariant::Xoroshiro128Plus
            | RngVariant::Xoshiro128 | RngVariant::Xoshiro128Plus => 16,
            RngVariant::Xoshiro256 | RngVariant::Xoshiro256Plus => 32,
            RngVariant::Xoshiro512 | RngVariant::Xoshiro512Plus => 64,
        }
    }

    /// Build a boxed `RngCore` from a seed buffer of the correct length
    #[allow(deprecated)] // RngCore is deprecated but still object‑safe and usable
    fn build_rng(&self, seed: &[u8]) -> Box<dyn RngCore + Send> {
        match self {
            RngVariant::Xoroshiro128 => {
                let arr: [u8; 16] = seed.try_into().unwrap();
                Box::new(Xoroshiro128StarStar::from_seed(arr))
            }
            RngVariant::Xoroshiro128Plus => {
                let arr: [u8; 16] = seed.try_into().unwrap();
                Box::new(Xoroshiro128Plus::from_seed(arr))
            }
            RngVariant::Xoshiro128 => {
                let arr: [u8; 16] = seed.try_into().unwrap();
                Box::new(Xoshiro128StarStar::from_seed(arr))
            }
            RngVariant::Xoshiro128Plus => {
                let arr: [u8; 16] = seed.try_into().unwrap();
                Box::new(Xoshiro128Plus::from_seed(arr))
            }
            RngVariant::Xoshiro256 => {
                let arr: [u8; 32] = seed.try_into().unwrap();
                Box::new(Xoshiro256StarStar::from_seed(arr))
            }
            RngVariant::Xoshiro256Plus => {
                let arr: [u8; 32] = seed.try_into().unwrap();
                Box::new(Xoshiro256Plus::from_seed(arr))
            }
            RngVariant::Xoshiro512 => {
                let arr: [u8; 64] = seed.try_into().unwrap();
                Box::new(Xoshiro512StarStar::from_seed(Seed512(arr)))
            }
            RngVariant::Xoshiro512Plus => {
                let arr: [u8; 64] = seed.try_into().unwrap();
                Box::new(Xoshiro512Plus::from_seed(Seed512(arr)))
            }
        }
    }
}

/// Command‑line arguments
#[derive(Parser)]
#[command(about = "Generate an infinite stream of random bytes using a xoroshiro/xoshiro PRNG.",
          group = ArgGroup::new("rng").multiple(false).required(false))]
struct Args {
    /// Select xoroshiro128** (default for 128‑bit)
    #[arg(long, group = "rng")]
    xoroshiro128: bool,

    /// Select xoroshiro128+
    #[arg(long, group = "rng")]
    xoroshiro128_plus: bool,

    /// Select xoshiro128** (128‑bit)
    #[arg(long, group = "rng")]
    xoshiro128: bool,

    /// Select xoshiro128+
    #[arg(long, group = "rng")]
    xoshiro128_plus: bool,

    /// Select xoshiro256** (default overall)
    #[arg(long, group = "rng")]
    xoshiro256: bool,

    /// Select xoshiro256+
    #[arg(long, group = "rng")]
    xoshiro256_plus: bool,

    /// Select xoshiro512**
    #[arg(long, group = "rng")]
    xoshiro512: bool,

    /// Select xoshiro512+
    #[arg(long, group = "rng")]
    xoshiro512_plus: bool,
}

impl Args {
    /// Determine which RNG variant was selected, defaulting to xoshiro256** if none.
    fn variant(&self) -> RngVariant {
        if self.xoroshiro128 {
            RngVariant::Xoroshiro128
        } else if self.xoroshiro128_plus {
            RngVariant::Xoroshiro128Plus
        } else if self.xoshiro128 {
            RngVariant::Xoshiro128
        } else if self.xoshiro128_plus {
            RngVariant::Xoshiro128Plus
        } else if self.xoshiro256 {
            RngVariant::Xoshiro256
        } else if self.xoshiro256_plus {
            RngVariant::Xoshiro256Plus
        } else if self.xoshiro512 {
            RngVariant::Xoshiro512
        } else if self.xoshiro512_plus {
            RngVariant::Xoshiro512Plus
        } else {
            // default if no flag is given
            RngVariant::Xoshiro256
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