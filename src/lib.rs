pub mod config;
pub mod tui;
pub mod app;
pub mod download;
pub mod database;

/// A function that generates a stable hash for `s`.
pub fn hash(s: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;

    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    hash.to_string()
}
