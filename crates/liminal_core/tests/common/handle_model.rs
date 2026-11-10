//! Test-local model of the handle rules (spec §10): the list of every issued handle, the list of
//! live handles and the list of freed handles, all searched linearly. Written from CORE-R033 to
//! CORE-R038, independent of `HandleAllocator`'s slot and free-list representation.
//!
//! Generation wrap (CORE-R035) is not modelled: reaching it needs 2^32 frees of one slot. It is
//! tested inside the crate.

use liminal_core::handle::RawHandle;

/// What `allocate` must return next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expected {
    Handle(RawHandle),
    Exhausted,
}

#[derive(Clone, Debug)]
pub struct HandleModel {
    slot_limit: u32,
    /// Every handle ever issued, in issue order.
    pub issued: Vec<RawHandle>,
    /// Handles issued and not freed since.
    pub live: Vec<RawHandle>,
    /// Freed handles whose slot was not reissued yet, in the order they were freed.
    freed: Vec<RawHandle>,
}

pub fn handle(index: u32, generation: u32) -> RawHandle {
    RawHandle::from_bits((u64::from(generation) << 32) | u64::from(index))
        .expect("test setup: generation must not be 0")
}

impl HandleModel {
    pub fn new(slot_limit: u32) -> HandleModel {
        HandleModel {
            slot_limit,
            issued: Vec::new(),
            live: Vec::new(),
            freed: Vec::new(),
        }
    }

    /// Number of distinct indices issued so far: the lowest never-used index (CORE-R033).
    fn indices_used(&self) -> u32 {
        let mut indices: Vec<u32> = self.issued.iter().map(|h| h.index()).collect();
        indices.sort_unstable();
        indices.dedup();
        u32::try_from(indices.len()).expect("test setup: fewer than 2^32 indices")
    }

    /// CORE-R033, CORE-R037: the most recently freed slot with its generation + 1, else the
    /// lowest never-used index with generation 1, else exhaustion.
    pub fn expected_allocation(&self) -> Expected {
        if let Some(freed) = self.freed.last() {
            return Expected::Handle(handle(freed.index(), freed.generation().get() + 1));
        }
        let index = self.indices_used();
        if index < self.slot_limit {
            Expected::Handle(handle(index, 1))
        } else {
            Expected::Exhausted
        }
    }

    /// Records the allocation that `expected_allocation` predicted.
    pub fn allocate(&mut self) -> Expected {
        let expected = self.expected_allocation();
        if let Expected::Handle(new) = expected {
            assert!(!self.issued.contains(&new), "model issued {new} twice");
            self.freed.pop();
            self.issued.push(new);
            self.live.push(new);
        }
        expected
    }

    pub fn is_live(&self, handle: RawHandle) -> bool {
        self.live.contains(&handle)
    }

    /// `true` if `handle` was live and is now freed; `false` (model unchanged) otherwise.
    pub fn free(&mut self, handle: RawHandle) -> bool {
        let Some(at) = self.live.iter().position(|live| *live == handle) else {
            return false;
        };
        self.live.remove(at);
        self.freed.push(handle);
        true
    }

    /// The live handles in ascending index order (CORE-R038).
    pub fn live_sorted(&self) -> Vec<RawHandle> {
        let mut live = self.live.clone();
        live.sort_by_key(|h| h.index());
        live
    }
}
