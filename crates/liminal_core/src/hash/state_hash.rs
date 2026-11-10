//! [`StateHash`]: the canonical encoding of values for the state hasher.

use super::StateHasher;
use crate::handle::{Handle, RawHandle};

/// The canonical state encoding of a value: `state_hash` writes the bytes that stand for the
/// value to a [`StateHasher`].
///
/// This is not `core::hash::Hash`: the `Hash` impls of `std` types are not guaranteed to stay
/// the same across Rust releases or platforms, so they cannot back recorded hashes.
///
/// Encodings of the implementations in this crate (spec CORE-R054):
///
/// | Type | Bytes written |
/// |---|---|
/// | `u8` … `u128`, `i8` … `i128` | little-endian two's complement, natural width |
/// | `usize`, `isize` | as `u64` / `i64` (8 bytes) |
/// | `bool` | one byte, `0x00` or `0x01` |
/// | `char` | the scalar value as `u32` |
/// | `f32`, `f64` | `to_bits()` as `u32` / `u64`; NaN payloads and `-0.0` are kept |
/// | `str`, `String` | length in bytes as `u64`, then the UTF-8 bytes |
/// | `[T]`, `Vec<T>`, `[T; N]` | length as `u64`, then each element in order |
/// | `Option<T>` | `0x00` for `None`; `0x01`, then the value, for `Some` |
/// | `&T` | as `T` |
/// | tuples of 1 to 4 elements | each element in order, no prefix |
/// | [`RawHandle`], [`Handle<T>`] | `to_bits()` as `u64` |
/// | [`DetHashMap`](crate::collections::DetHashMap) | length as `u64`, then each key and value in iteration order |
/// | [`DetHashSet`](crate::collections::DetHashSet) | length as `u64`, then each value in iteration order |
/// | [`Rng`](crate::rng::Rng) | the four state words as `u64`, in order |
/// | [`StreamKey`](crate::rng::StreamKey) | the value as `u64` |
/// | [`Tick`](crate::time::Tick), [`TickDuration`](crate::time::TickDuration) | the count as `u64` |
/// | [`TickRate`](crate::time::TickRate) | ticks per second as `u32` |
///
/// An implementation for another type MUST write the same bytes for equal values and SHOULD
/// write different bytes for unequal values: hash every field that is part of the state, in a
/// fixed order, and prefix variable-length parts with their length and enum variants with a tag.
///
/// # Examples
///
/// ```
/// use liminal_core::hash::{StateHash, StateHasher, state_hash_of};
///
/// struct Body {
///     position: [f32; 3],
///     name: String,
/// }
///
/// impl StateHash for Body {
///     fn state_hash(&self, hasher: &mut StateHasher) {
///         self.position.state_hash(hasher);
///         self.name.state_hash(hasher);
///     }
/// }
///
/// let body = Body { position: [1.0, 2.0, 3.0], name: "crate".to_string() };
/// let same = Body { position: [1.0, 2.0, 3.0], name: "crate".to_string() };
/// let moved = Body { position: [1.0, 2.5, 3.0], name: "crate".to_string() };
/// assert_eq!(state_hash_of(&body), state_hash_of(&same));
/// assert_ne!(state_hash_of(&body), state_hash_of(&moved));
/// ```
pub trait StateHash {
    /// Writes the canonical encoding of `self` to `hasher`.
    fn state_hash(&self, hasher: &mut StateHasher);
}

/// The state hash of one value: `finish()` of a new [`StateHasher`] after
/// `value.state_hash(&mut hasher)`.
///
/// # Examples
///
/// ```
/// use liminal_core::hash::{StateHasher, state_hash_of};
///
/// let scores = vec![3_u32, 1, 2];
/// let hash = state_hash_of(&scores);
///
/// // A `Vec` writes its length as `u64`, then each element.
/// let mut hasher = StateHasher::new();
/// hasher.write_u64(3);
/// for score in [3_u32, 1, 2] {
///     hasher.write_u32(score);
/// }
/// assert_eq!(hash, hasher.finish());
///
/// // Order is part of the state.
/// assert_ne!(hash, state_hash_of(&vec![1_u32, 2, 3]));
/// ```
#[must_use]
pub fn state_hash_of<T: StateHash + ?Sized>(value: &T) -> u64 {
    let mut hasher = StateHasher::new();
    value.state_hash(&mut hasher);
    hasher.finish()
}

/// Implements `StateHash` for `Copy` types through the hasher method of the same encoding.
macro_rules! state_hash_by_method {
    ($($type:ty => $method:ident;)*) => {
        $(
            impl StateHash for $type {
                fn state_hash(&self, hasher: &mut StateHasher) {
                    hasher.$method(*self);
                }
            }
        )*
    };
}

state_hash_by_method! {
    u8 => write_u8;
    u16 => write_u16;
    u32 => write_u32;
    u64 => write_u64;
    u128 => write_u128;
    usize => write_usize;
    i8 => write_i8;
    i16 => write_i16;
    i32 => write_i32;
    i64 => write_i64;
    i128 => write_i128;
    isize => write_isize;
    bool => write_bool;
    char => write_char;
    f32 => write_f32;
    f64 => write_f64;
}

impl StateHash for str {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_str(self);
    }
}

impl StateHash for String {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_str(self);
    }
}

impl<T: StateHash> StateHash for [T] {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_usize(self.len());
        for element in self {
            element.state_hash(hasher);
        }
    }
}

impl<T: StateHash> StateHash for Vec<T> {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.as_slice().state_hash(hasher);
    }
}

impl<T: StateHash, const N: usize> StateHash for [T; N] {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.as_slice().state_hash(hasher);
    }
}

impl<T: StateHash> StateHash for Option<T> {
    fn state_hash(&self, hasher: &mut StateHasher) {
        match self {
            None => hasher.write_u8(0),
            Some(value) => {
                hasher.write_u8(1);
                value.state_hash(hasher);
            }
        }
    }
}

impl<T: StateHash + ?Sized> StateHash for &T {
    fn state_hash(&self, hasher: &mut StateHasher) {
        (**self).state_hash(hasher);
    }
}

/// Implements `StateHash` for a tuple: each element in order, no prefix.
macro_rules! state_hash_tuple {
    ($($name:ident: $index:tt),+) => {
        impl<$($name: StateHash),+> StateHash for ($($name,)+) {
            fn state_hash(&self, hasher: &mut StateHasher) {
                $(self.$index.state_hash(hasher);)+
            }
        }
    };
}

state_hash_tuple!(A: 0);
state_hash_tuple!(A: 0, B: 1);
state_hash_tuple!(A: 0, B: 1, C: 2);
state_hash_tuple!(A: 0, B: 1, C: 2, D: 3);

impl StateHash for RawHandle {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_u64(self.to_bits());
    }
}

impl<T> StateHash for Handle<T> {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.raw().state_hash(hasher);
    }
}
