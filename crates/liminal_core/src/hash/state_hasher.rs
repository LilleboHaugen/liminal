//! [`StateHasher`]: the streaming state hasher.

use super::xxh64::{Xxh64, len_u64};

/// The state hasher: XXH64 with seed 0 over the bytes written to it (spec CORE-R050 to
/// CORE-R057).
///
/// Every write appends bytes to one sequence, and [`finish`](Self::finish) returns XXH64 of that
/// sequence. Only the byte sequence matters: splitting it into different writes gives the same
/// hash. The typed methods append a fixed encoding (integers little-endian in their natural
/// width, `usize` and `isize` as 64 bits), so the same values give the same hash on every
/// supported platform and in every process.
///
/// Fixed-width writes add no separators. Variable-length data needs a length prefix so that
/// different values cannot give the same byte sequence: [`write_str`](Self::write_str) and the
/// [`StateHash`](super::StateHash) impls of slices and collections write one;
/// [`write_bytes`](Self::write_bytes) does not.
///
/// Not a cryptographic hash, and not a `core::hash::Hasher` (see [`DetHasher`](super::DetHasher)
/// for that).
///
/// # Examples
///
/// ```
/// use liminal_core::hash::StateHasher;
///
/// let mut hasher = StateHasher::new();
/// hasher.write_u32(7);
/// hasher.write_str("player");
/// hasher.write_f32(1.5);
/// let hash = hasher.finish();
///
/// // The same writes give the same hash, in every process and on every platform.
/// let mut again = StateHasher::new();
/// again.write_u32(7);
/// again.write_str("player");
/// again.write_f32(1.5);
/// assert_eq!(again.finish(), hash);
///
/// // Only the bytes count: `write_u32(7)` appends `07 00 00 00`.
/// let mut bytes = StateHasher::new();
/// bytes.write_bytes(&[7, 0, 0, 0]);
/// bytes.write_bytes(&6_u64.to_le_bytes()); // the length prefix of "player"
/// bytes.write_bytes(b"player");
/// bytes.write_bytes(&1.5_f32.to_bits().to_le_bytes());
/// assert_eq!(bytes.finish(), hash);
///
/// // XXH64 (seed 0) of the empty input.
/// assert_eq!(StateHasher::new().finish(), 0xEF46_DB37_51D8_E999);
/// ```
#[derive(Clone, Debug)]
pub struct StateHasher {
    state: Xxh64,
}

/// Defines typed write methods that append the value's little-endian bytes.
macro_rules! write_le_methods {
    ($($name:ident: $type:ty, $what:literal;)*) => {
        $(
            #[doc = concat!("Appends ", $what, ".")]
            pub fn $name(&mut self, value: $type) {
                self.state.write(&value.to_le_bytes());
            }
        )*
    };
}

impl StateHasher {
    /// A hasher with nothing written. Its `finish` is XXH64 (seed 0) of the empty input,
    /// `0xEF46_DB37_51D8_E999`.
    #[must_use]
    pub fn new() -> StateHasher {
        StateHasher {
            state: Xxh64::new(),
        }
    }

    /// Appends the bytes as they are, with no length prefix.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.state.write(bytes);
    }

    write_le_methods! {
        write_u8: u8, "one byte";
        write_u16: u16, "the 2 little-endian bytes of the value";
        write_u32: u32, "the 4 little-endian bytes of the value";
        write_u64: u64, "the 8 little-endian bytes of the value";
        write_u128: u128, "the 16 little-endian bytes of the value";
        write_i8: i8, "one byte, the two's-complement form of the value";
        write_i16: i16, "the 2 little-endian two's-complement bytes of the value";
        write_i32: i32, "the 4 little-endian two's-complement bytes of the value";
        write_i64: i64, "the 8 little-endian two's-complement bytes of the value";
        write_i128: i128, "the 16 little-endian two's-complement bytes of the value";
    }

    /// Appends the value as a `u64` (8 little-endian bytes), whatever the width of `usize`.
    pub fn write_usize(&mut self, value: usize) {
        self.write_u64(len_u64(value));
    }

    /// Appends the value as an `i64` (8 little-endian two's-complement bytes), whatever the
    /// width of `isize`.
    ///
    /// # Panics
    ///
    /// Only on a target whose `isize` is wider than 64 bits; no supported target is.
    pub fn write_isize(&mut self, value: isize) {
        self.write_i64(i64::try_from(value).expect("invariant: isize is at most 64 bits"));
    }

    /// Appends one byte: `0x00` for `false`, `0x01` for `true`.
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    /// Appends the scalar value of the character as a `u32` (4 little-endian bytes).
    pub fn write_char(&mut self, value: char) {
        self.write_u32(u32::from(value));
    }

    /// Appends the 4 little-endian bytes of `value.to_bits()`. NaN payloads and the sign of zero
    /// are kept: `0.0` and `-0.0` hash differently, and so do two NaNs with different bits.
    pub fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    /// Appends the 8 little-endian bytes of `value.to_bits()`. NaN payloads and the sign of zero
    /// are kept: `0.0` and `-0.0` hash differently, and so do two NaNs with different bits.
    pub fn write_f64(&mut self, value: f64) {
        self.write_u64(value.to_bits());
    }

    /// Appends the length of the string in bytes as a `u64`, then its UTF-8 bytes.
    pub fn write_str(&mut self, value: &str) {
        self.write_usize(value.len());
        self.write_bytes(value.as_bytes());
    }

    /// XXH64 (seed 0) of all bytes written so far. Does not change the hasher: calling it again
    /// returns the same value, and later writes continue the same byte sequence.
    #[must_use]
    pub fn finish(&self) -> u64 {
        self.state.finish()
    }
}

/// `new()`: a hasher with nothing written.
impl Default for StateHasher {
    fn default() -> StateHasher {
        StateHasher::new()
    }
}
