use anyhow::{Context, Result};
use std::fs;
use std::mem;
use std::path::Path;

pub const TABLE_ENTRIES: usize = 65536;
pub const NUM_TABLES: usize = 4;
const ENTRY_SIZE: usize = mem::size_of::<u64>();

pub const MAP_SIZE: usize = TABLE_ENTRIES * NUM_TABLES * ENTRY_SIZE;
const PRIME_STEP: u64 = 7776210437768060567;

pub type Q64 = [u8; ENTRY_SIZE];

/// A validated 2 MiB seed map for QXO64.
#[derive(Clone, Debug)]
pub struct Q64Map([u8; MAP_SIZE]);

impl Q64Map {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let arr: [u8; MAP_SIZE] = bytes
            .try_into()
            .context("Q64Map: invalid seed length")?;
        Ok(Q64Map(arr))
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        Self::from_bytes(&fs::read(path)?)
    }

    /// Generate a fresh chaotic map using the Seedy64 entropy source.
    pub fn generate_with_seedy() -> Result<Self> {
        let mut arr = [0u8; MAP_SIZE];
        crate::seedy64::Seedy64::new().fill_buffer(&mut arr);
        Ok(Q64Map(arr))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir)?;
        fs::write(path, &self.0)?;
        Ok(())
    }

    pub fn as_bytes(&self) -> &[u8; MAP_SIZE] {
        &self.0
    }
}

pub struct Qxo64 {
    tables: Box<[[u64; TABLE_ENTRIES]; NUM_TABLES]>,
}

impl Qxo64 {
    pub fn new(map: Q64Map) -> Self {
        let words: &[u64] = bytemuck::cast_slice(map.as_bytes());
        let mut tables = Box::new([[0u64; TABLE_ENTRIES]; NUM_TABLES]);
        for (i, chunk) in words.chunks_exact(TABLE_ENTRIES).enumerate() {
            tables[i].copy_from_slice(chunk);
        }
        Qxo64 { tables }
    }

    pub fn load(path: &Path) -> Result<Self> {
        Ok(Self::new(Q64Map::from_file(path)?))
    }

    pub fn generate_and_save(path: &Path) -> Result<Self> {
        let map = Q64Map::generate_with_seedy()?;
        map.save(path)?;
        Ok(Self::new(map))
    }

    pub fn generate_and_save_with_seed(path: &Path, seed_file: &Path) -> Result<Self> {
        let map = Q64Map::from_file(seed_file)?;
        map.save(path)?;
        Ok(Self::new(map))
    }

    #[inline]
    pub fn read_chunk(&self, idx: u64) -> Q64 {
        let counter = idx.wrapping_mul(PRIME_STEP);
        let i0 = (counter & 0xFFFF) as usize;
        let i1 = ((counter >> 16) & 0xFFFF) as usize;
        let i2 = ((counter >> 32) & 0xFFFF) as usize;
        let i3 = ((counter >> 48) & 0xFFFF) as usize;
        let val = self.tables[0][i0] ^ self.tables[1][i1] ^ self.tables[2][i2] ^ self.tables[3][i3];
        val.to_le_bytes()
    }

    pub fn fill_chunks(&self, buffer: &mut [Q64], start_idx: u64) {
        let mut idx = start_idx;
        for chunk in buffer.iter_mut() {
            *chunk = self.read_chunk(idx);
            idx = idx.wrapping_add(1);
        }
    }
}