//! The lookup structure of [`DetHashMap`](super::DetHashMap): an open-addressing table that maps
//! a key's hash to the position of its entry in the map's ordered entry list.

/// Marks a cell that holds no entry position.
const EMPTY: usize = usize::MAX;

/// Smallest number of cells of a table that holds anything.
const MIN_CELLS: usize = 8;

/// Open-addressing hash table with linear probing. A cell is `EMPTY` or the position of an entry
/// in the owner's entry list. The table never stores keys: the owner compares them.
///
/// The number of cells is 0 or a power of two and at least twice the number of occupied cells,
/// so every probe sequence reaches an `EMPTY` cell. Cells are never deleted one by one (no
/// tombstones): the owner rebuilds the table when an entry is removed.
#[derive(Clone, Debug)]
pub(super) struct IndexTable {
    cells: Vec<usize>,
}

impl IndexTable {
    /// A table with no cells. Allocates nothing.
    pub(super) const fn new() -> IndexTable {
        IndexTable { cells: Vec::new() }
    }

    /// An empty table that holds `entries` positions without growing.
    pub(super) fn with_capacity(entries: usize) -> IndexTable {
        let mut table = IndexTable::new();
        if entries > 0 {
            table.cells = vec![EMPTY; cells_for(entries)];
        }
        table
    }

    /// `true` if the table can hold `entries` positions without growing.
    pub(super) fn holds(&self, entries: usize) -> bool {
        entries == 0 || cells_for(entries) <= self.cells.len()
    }

    /// The position stored for `hash` for which `is_match` is true, if any. `is_match` is called
    /// with each candidate position (entries whose hash selects the same probe sequence).
    pub(super) fn find(&self, hash: u64, mut is_match: impl FnMut(usize) -> bool) -> Option<usize> {
        self.probe(hash)
            .map_while(|cell| {
                self.cells
                    .get(cell)
                    .copied()
                    .filter(|&position| position != EMPTY)
            })
            .find(|&position| is_match(position))
    }

    /// Stores `position` for `hash`. The caller has checked that no stored position matches the
    /// key and that the table [`holds`](Self::holds) one more entry.
    pub(super) fn insert(&mut self, hash: u64, position: usize) {
        debug_assert!(
            position != EMPTY,
            "INV-3: an entry position equals the empty marker"
        );
        let free = self
            .probe(hash)
            .find(|&cell| self.cells.get(cell) == Some(&EMPTY));
        debug_assert!(free.is_some(), "INV-3: the index table has no free cell");
        if let Some(cell) = free.and_then(|cell| self.cells.get_mut(cell)) {
            *cell = position;
        }
    }

    /// Empties the table and stores positions `0, 1, 2, …` for `hashes` in order, in a table
    /// that holds at least `capacity` entries (and at least the entries given).
    pub(super) fn rebuild(&mut self, capacity: usize, hashes: impl ExactSizeIterator<Item = u64>) {
        let entries = capacity.max(hashes.len());
        if self.holds(entries) {
            self.cells.fill(EMPTY);
        } else {
            self.cells = vec![EMPTY; cells_for(entries)];
        }
        for (position, hash) in hashes.enumerate() {
            self.insert(hash, position);
        }
    }

    /// Marks every cell empty and keeps the allocation.
    pub(super) fn clear(&mut self) {
        self.cells.fill(EMPTY);
    }

    /// Number of occupied cells. O(cells); for invariant checks.
    pub(super) fn occupied(&self) -> usize {
        self.cells.iter().filter(|&&cell| cell != EMPTY).count()
    }

    /// The cells to inspect for `hash`, in order: each cell once, starting at the cell the hash
    /// selects. Yields nothing for a table with no cells.
    fn probe(&self, hash: u64) -> impl Iterator<Item = usize> + use<> {
        let cells = self.cells.len();
        let mask = cells.wrapping_sub(1);
        let start = low_bits(hash) & mask;
        (0..cells).map(move |step| start.wrapping_add(step) & mask)
    }
}

/// Number of cells for `entries` positions: a power of two, at least `MIN_CELLS` and at least
/// `2 × entries` (load factor at most 1/2).
fn cells_for(entries: usize) -> usize {
    entries
        .saturating_mul(2)
        .max(MIN_CELLS)
        .checked_next_power_of_two()
        .expect("invariant: an entry list of this length cannot be allocated")
}

/// The low bits of a hash as `usize`; the caller masks them to select a cell.
#[expect(
    clippy::cast_possible_truncation,
    reason = "only the low bits select a cell; dropping high bits on a narrower usize is intended"
)]
fn low_bits(hash: u64) -> usize {
    hash as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_table_without_cells_finds_nothing_and_holds_nothing() {
        let table = IndexTable::new();
        assert_eq!(table.find(7, |_| true), None);
        assert!(table.holds(0));
        assert!(!table.holds(1));
        assert_eq!(table.occupied(), 0);
    }

    #[test]
    fn colliding_hashes_are_told_apart_by_the_match_predicate() {
        let mut table = IndexTable::with_capacity(4);
        // Hashes 3 and 11 select the same cell of an 8-cell table; 7 wraps around the end.
        table.insert(3, 0);
        table.insert(11, 1);
        table.insert(7, 2);
        table.insert(15, 3);
        assert_eq!(table.occupied(), 4);
        assert_eq!(table.find(3, |position| position == 0), Some(0));
        assert_eq!(table.find(11, |position| position == 1), Some(1));
        assert_eq!(table.find(15, |position| position == 3), Some(3));
        assert_eq!(table.find(11, |_| false), None);
        // Hash 4 starts at the cell the collision pushed position 1 into; hash 5 at a free one.
        assert_eq!(table.find(4, |_| true), Some(1));
        assert_eq!(table.find(5, |_| true), None);
    }

    #[test]
    fn rebuild_grows_to_the_requested_capacity_and_renumbers_positions() {
        let mut table = IndexTable::with_capacity(1);
        assert!(table.holds(4) && !table.holds(5));
        table.rebuild(100, [42_u64, 43].into_iter());
        assert!(table.holds(100));
        assert_eq!(table.find(42, |_| true), Some(0));
        assert_eq!(table.find(43, |_| true), Some(1));
        table.rebuild(0, [43_u64].into_iter());
        assert!(table.holds(100), "a rebuild never shrinks the table");
        assert_eq!(table.find(42, |_| true), None);
        assert_eq!(table.find(43, |_| true), Some(0));
        table.clear();
        assert_eq!(table.occupied(), 0);
        assert!(table.holds(100));
    }
}
