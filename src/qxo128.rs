use anyhow::{Context, Result};
use std::fs;
use std::mem;
use std::path::Path;

pub const TABLE_ENTRIES: usize = 65536;
pub const NUM_TABLES: usize = 8;
const ENTRY_SIZE: usize = mem::size_of::<u64>();

pub const MAP_SIZE: usize = TABLE_ENTRIES * NUM_TABLES * ENTRY_SIZE;
const PRIME: u128 = 0xed07c5d781435a8588ebabd3a0753339;

pub type Q128 = [u8; 2 * ENTRY_SIZE];

/// A validated 4 MiB seed map for QXO128.
#[derive(Clone, Debug)]
pub struct Q128Map([u8; MAP_SIZE]);

impl Q128Map {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let arr: [u8; MAP_SIZE] = bytes
            .try_into()
            .context("Q128Map: invalid seed length")?;
        Ok(Q128Map(arr))
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        Self::from_bytes(&fs::read(path)?)
    }

    /// Generate a fresh chaotic map using the Seedy64 entropy source.
    pub fn generate_with_seedy() -> Result<Self> {
        let mut arr = [0u8; MAP_SIZE];
        crate::seedy64::Seedy64::new().fill_buffer(&mut arr);
        Ok(Q128Map(arr))
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

pub struct Qxo128 {
    tables: Box<[[u64; TABLE_ENTRIES]; NUM_TABLES]>,
}

impl Qxo128 {
    pub fn new(map: Q128Map) -> Self {
        let words: &[u64] = bytemuck::cast_slice(map.as_bytes());
        let mut tables = Box::new([[0u64; TABLE_ENTRIES]; NUM_TABLES]);
        for (i, chunk) in words.chunks_exact(TABLE_ENTRIES).enumerate() {
            tables[i].copy_from_slice(chunk);
        }
        Qxo128 { tables }
    }

    pub fn load(path: &Path) -> Result<Self> {
        Ok(Self::new(Q128Map::from_file(path)?))
    }

    pub fn generate_and_save(path: &Path) -> Result<Self> {
        let map = Q128Map::generate_with_seedy()?;
        map.save(path)?;
        Ok(Self::new(map))
    }

    pub fn generate_and_save_with_seed(path: &Path, seed_file: &Path) -> Result<Self> {
        let map = Q128Map::from_file(seed_file)?;
        map.save(path)?;
        Ok(Self::new(map))
    }

    #[inline]
    pub fn read_chunk(&self, idx: u128) -> Q128 {
        let mixed = idx.wrapping_mul(PRIME);
        let low0 = (mixed & 0xFFFF) as usize;
        let low1 = ((mixed >> 16) & 0xFFFF) as usize;
        let low2 = ((mixed >> 32) & 0xFFFF) as usize;
        let low3 = ((mixed >> 48) & 0xFFFF) as usize;
        let high0 = ((mixed >> 64) & 0xFFFF) as usize;
        let high1 = ((mixed >> 80) & 0xFFFF) as usize;
        let high2 = ((mixed >> 96) & 0xFFFF) as usize;
        let high3 = ((mixed >> 112) & 0xFFFF) as usize;

        let low = self.tables[0][low0] ^ self.tables[1][low1] ^ self.tables[2][low2] ^ self.tables[3][low3];
        let high = self.tables[4][high0] ^ self.tables[5][high1] ^ self.tables[6][high2] ^ self.tables[7][high3];

        let mut out = [0u8; 2 * ENTRY_SIZE];
        out[0..ENTRY_SIZE].copy_from_slice(&low.to_le_bytes());
        out[ENTRY_SIZE..].copy_from_slice(&high.to_le_bytes());
        out
    }

    pub fn fill_chunks(&self, buffer: &mut [Q128], start_idx: u128) {
        let mut idx = start_idx;
        for chunk in buffer.iter_mut() {
            *chunk = self.read_chunk(idx);
            idx = idx.wrapping_add(1);
        }
    }
}