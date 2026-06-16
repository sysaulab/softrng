use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub const TABLE_ENTRIES: usize = 65536;       // 2¹⁶
pub const NUM_TABLES: usize = 4;
pub const MAP_SIZE: usize = TABLE_ENTRIES * NUM_TABLES * 8; // 2 MiB
const PRIME_STEP: u64 = 7776210437768060567;

pub struct Qxo64 {
    tables: Vec<u64>,
}

impl Qxo64 {
    /// Load from a raw byte slice.
    pub fn new_from_data(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != MAP_SIZE {
            anyhow::bail!("invalid seed length: expected {MAP_SIZE}, got {}", bytes.len());
        }

        // Interpret as little‑endian u64 – this is fine because endianness
        // doesn't matter for the output; we just keep whatever the file gives us.
        let mut tables = vec![0u64; TABLE_ENTRIES * NUM_TABLES];
        let dest_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                tables.as_mut_ptr() as *mut u8,
                MAP_SIZE,
            )
        };
        dest_bytes.copy_from_slice(bytes);

        Ok(Qxo64 { tables })
    }

    /// Load from a file.
    pub fn new_from_file(path: &Path) -> Result<Self> {
        Self::new_from_data(&fs::read(path)?)
    }

    /// Return the next native‑endian u64 for index `idx`.
    #[inline]
    pub fn read_chunk(&self, idx: u64) -> u64 {
        let counter = idx.wrapping_mul(PRIME_STEP);
        let i0 = (counter & 0xFFFF) as usize;
        let i1 = ((counter >> 16) & 0xFFFF) as usize;
        let i2 = ((counter >> 32) & 0xFFFF) as usize;
        let i3 = ((counter >> 48) & 0xFFFF) as usize;

        self.tables[i0]
            ^ self.tables[TABLE_ENTRIES + i1]
            ^ self.tables[2 * TABLE_ENTRIES + i2]
            ^ self.tables[3 * TABLE_ENTRIES + i3]
    }

    /// Fill a slice of `u64` words.
    pub fn fill_u64s(&self, buffer: &mut [u64], start_idx: u64) {
        let mut idx = start_idx;
        for word in buffer.iter_mut() {
            *word = self.read_chunk(idx);
            idx = idx.wrapping_add(1);
        }
    }

    /// Fill a byte buffer starting at absolute byte offset `byte_offset`.
    pub fn fill_bytes(&self, buffer: &mut [u8], byte_offset: u128) {
        if buffer.is_empty() {
            return;
        }

        // Convert byte offset to chunk index and intra‑chunk position.
        let start_chunk_idx = (byte_offset / 8) as u64;
        let byte_in_chunk = (byte_offset % 8) as usize;

        let mut chunk_idx = start_chunk_idx;
        let mut chunk = self.read_chunk(chunk_idx);
        let chunk_bytes = chunk.to_ne_bytes();

        // First (possibly partial) chunk.
        let take_first = 8usize.saturating_sub(byte_in_chunk).min(buffer.len());
        buffer[..take_first].copy_from_slice(&chunk_bytes[byte_in_chunk..byte_in_chunk + take_first]);
        let mut written = take_first;

        // Full 8‑byte chunks.
        while buffer.len() - written >= 8 {
            chunk_idx = chunk_idx.wrapping_add(1);
            chunk = self.read_chunk(chunk_idx);
            buffer[written..written + 8].copy_from_slice(&chunk.to_ne_bytes());
            written += 8;
        }

        // Final partial chunk.
        if written < buffer.len() {
            chunk_idx = chunk_idx.wrapping_add(1);
            chunk = self.read_chunk(chunk_idx);
            let remaining = buffer.len() - written;
            buffer[written..].copy_from_slice(&chunk.to_ne_bytes()[..remaining]);
        }
    }
}