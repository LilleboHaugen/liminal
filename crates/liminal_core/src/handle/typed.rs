//! [`Handle<T>`]: a raw handle with the type of the value it names.

use core::cmp::Ordering;
use core::marker::PhantomData;

use super::RawHandle;

/// Typed handle: a [`RawHandle`] that names a value of type `T`, so a handle to one kind of
/// value cannot be passed where another kind is expected.
///
/// 8 bytes, like `Option<Handle<T>>`. `Copy`, `Eq`, `Ord`, `Hash` and `Debug` for every `T`,
/// with no bounds on `T`; equality and order are those of the raw handles. Issued by
/// [`HandleMap`](super::HandleMap).
///
/// # Examples
///
/// ```
/// use liminal_core::error::CoreError;
/// use liminal_core::handle::{Handle, HandleMap};
///
/// struct Texture; // implements no trait at all
///
/// let mut textures: HandleMap<Texture> = HandleMap::new();
/// let handle: Handle<Texture> = textures.insert(Texture)?;
/// let copy = handle; // `Handle<Texture>` is `Copy` although `Texture` is not
/// assert_eq!(handle, copy);
/// assert_eq!(Handle::<Texture>::from_raw(handle.raw()), handle);
/// # Ok::<(), CoreError>(())
/// ```
pub struct Handle<T> {
    raw: RawHandle,
    // `fn() -> T`: the handle owns no `T`, and is `Send` and `Sync` whatever `T` is.
    marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    /// The typed handle for `raw`. The caller decides that `raw` names a `T`; nothing checks it.
    #[must_use]
    pub fn from_raw(raw: RawHandle) -> Handle<T> {
        Handle {
            raw,
            marker: PhantomData,
        }
    }

    /// The untyped handle.
    #[must_use]
    pub fn raw(self) -> RawHandle {
        self.raw
    }
}

// Implemented by hand: derives would add `T: Clone`, `T: PartialEq`, … bounds.

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Handle<T> {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Handle<T>) -> bool {
        self.raw == other.raw
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Handle<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Handle<T>) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> core::hash::Hash for Handle<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

/// Writes `Handle(` and the raw handle's `Debug` form, without naming `T`.
impl<T> core::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Handle").field(&self.raw).finish()
    }
}
