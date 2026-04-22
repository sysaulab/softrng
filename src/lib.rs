// src/lib.rs
pub mod seedy64;
pub mod qxo64;
pub mod qxo128;

// Re-export the main types for convenience
//pub use libseedy64::Seedy64;
//pub use libqxo64::Qxo64;
//pub use libqxo128::Qxo128;

// Buffer sizes (match original C sr_config.h values)
pub const BUFFER_SIZE: usize = 64 * 1024;      // typical _SSRNG_BUFSIZE
pub const BUFLEN: usize = BUFFER_SIZE / 8;     // for u64 chunks (as in f-merge)
//pub const FPS: f64 = 10.0;                     // update frequency for f-peek

// Precomputed hex characters for fast byte‑to‑hex conversion.
pub const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

// Convert a byte slice to a hex string (lowercase) efficiently.
//#[inline]
//pub fn bytes_to_hex(bytes: &[u8]) -> String {
//    let mut hex = vec![0u8; bytes.len() * 2];
//    for (i, &b) in bytes.iter().enumerate() {
//        hex[i * 2] = HEX_CHARS[(b >> 4) as usize];
//        hex[i * 2 + 1] = HEX_CHARS[(b & 0x0f) as usize];
//    }
//    unsafe { String::from_utf8_unchecked(hex) }
//}