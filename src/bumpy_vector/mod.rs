//#![feature(range_is_empty)]
//! [![Crate](https://img.shields.io/crates/v/bumpy_vector.svg)](https://crates.io/crates/bumpy_vector)
//!
//! A vector-like object where elements can be larger than one item. We use
//! this primarily to represent objects in a binary that are made up of one
//! or more bytes.
//!
//! # Goal
//!
//! [h2gb](https://github.com/h2gb/libh2gb) is a tool for analyzing binary
//! files. Importantly, a binary file is a series of objects, each of which
//! take up some number of bytes. We need a datatype to represent this unusual
//! requirement, hence coming up with BumpyVector!
//!
//! # Usage
//!
//! Instantiate with a maximum size, then use somewhat like a vector:
//!
//! ```
//! use libh2gb::bumpy_vector::{BumpyEntry, BumpyVector};
//!
//! // Instantiate with a maximum size of 100 and a type of String
//! let mut v: BumpyVector<String> = BumpyVector::new(100);
//!
//! // Create a 10-byte entry at the start
//! let entry: BumpyEntry<String> = BumpyEntry {
//!   entry: String::from("hello"),
//!   range: 0..10,
//! };
//!
//! // Insert it into the BumpyVector
//! assert!(v.insert(entry).is_ok());
//!
//! // Create another entry, this time from a tuple, that overlaps the first
//! let entry: BumpyEntry<String> = (String::from("error"), 1..6).into();
//! assert!(v.insert(entry).is_err());
//!
//! // Create an entry that's off the end of the object
//! let entry: BumpyEntry<String> = (String::from("error"), 1000..1005).into();
//! assert!(v.insert(entry).is_err());
//!
//! // There is still one entry in this vector
//! assert_eq!(1, v.len());
//! ```
//!
//! # Serialize / deserialize
//!
//! Serialization support using [serde](https://serde.rs/) is enabled. The
//! `BumpyVector` can be serialized with any of the serializers that Serde
//! supports, such as [ron](https://github.com/ron-rs/ron):
//!
//! ```
//! use libh2gb::bumpy_vector::BumpyVector;
//!
//! fn main() {
//!     let mut h: BumpyVector<String> = BumpyVector::new(10);
//!     h.insert((String::from("a"), 1..3).into()).unwrap();
//!
//!     // Serialize
//!     let serialized = ron::ser::to_string(&h).unwrap();
//!
//!     // Deserialize
//!     let h: BumpyVector<String> = ron::de::from_str(&serialized).unwrap();
//! }
//! ```

use std::collections::HashMap;
use std::ops::Range;

use simple_error::{SimpleResult, bail};

use serde::{Serialize, Deserialize};

/// Represents a single entry.
///
/// An entry is comprised of an object of type `T`, and a
/// [range](https://doc.rust-lang.org/std/ops/struct.Range.html). The range is
/// normal for Rust's `std::op::Range`, which means it's:
///
/// > A (half-open) range bounded inclusively below and exclusively above (start..end).
///
/// That means it starts at `start`, and ends at `end - 1`. The size in elements
/// is `end - start`.
///
/// # Example 1
///
/// Creating a basic entry is very straight forward:
///
/// ```
/// use libh2gb::bumpy_vector::BumpyEntry;
///
/// let e: BumpyEntry<&str> = BumpyEntry {
///   entry: "hello",
///   range: 0..1,
/// };
/// ```
///
/// # Example 2
///
/// For convenience, you can create an entry from a (T, Range) tuple:
///
/// ```
/// use libh2gb::bumpy_vector::BumpyEntry;
///
/// let e: BumpyEntry<&str> = ("hello", 0..1).into();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BumpyEntry<T> {
    pub entry: T,
    pub range: Range<usize>,
}

// Lets us store an entry, but also store a pointer to a non-entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaBumpyEntry<T> {
    Something(BumpyEntry<T>),
    NearlySomething(usize),
}

/// Implemented by a type that knows how to be a BumpyEntry.
///
/// That is to say, a type that has a built-in index and size, which can be
/// converted into a `Range`.
///
/// # Example
///
/// ```
/// use libh2gb::bumpy_vector::{BumpyVector, AutoBumpyEntry};
/// use std::ops::Range;
///
/// // A struct that has an index and size, then whatever other data
/// struct MyAutoEntry {
///     size: usize,
///     index: usize,
///     // ...anything else
/// };
///
/// // Simplest possible implementation for MyAutoEntry
/// impl AutoBumpyEntry for MyAutoEntry {
///     fn range(&self) -> Range<usize> {
///         self.index..(self.index + self.size)
///     }
/// };
///
/// // Create an entry
/// let entry = MyAutoEntry {
///     index: 0,
///     size: 10,
/// };
///
/// // Create a BumpyVector with the type we created
/// let mut h: BumpyVector<MyAutoEntry> = BumpyVector::new(100);
///
/// // Insert it
/// assert!(h.insert_auto(entry).is_ok());
/// ```

pub trait AutoBumpyEntry {
    fn range(&self) -> Range<usize>;
}

impl<T> From<(T, Range<usize>)> for BumpyEntry<T> {
    fn from(o: (T, Range<usize>)) -> Self {
        BumpyEntry {
          entry: o.0,
          range: o.1.clone(),
        }
    }
}

impl<T> From<T> for BumpyEntry<T>
where
    T: AutoBumpyEntry
{
    fn from(o: T) -> Self
    {
        // Pull this out before putting the entry in the list
        let range = o.range().clone();

        // Just use the built-in range. There might be a better way to do this?
        BumpyEntry {
          entry: o,
          range: range,
        }
    }
}

/// Represents an instance of a Bumpy Vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BumpyVector<T> {
    /// The data is represented by a HashMap, where the index is the key and
    /// a BumpyEntry is the object.
    data: HashMap<usize, MetaBumpyEntry<T>>,

    /// The maximum size.
    max_size: usize,
}

/// Implement the object.
impl<'a, T> BumpyVector<T> {
    /// Create a new instance of BumpyVector.
    ///
    /// The range of the vector goes from `0` to `max_size - 1`. If any
    /// elements beyond the end are accessed, an error will be returned.
    pub fn new(max_size: usize) -> Self {
        BumpyVector {
            data: HashMap::new(),
            max_size: max_size,
        }
    }

    /// Get the object that starts at or overlaps the starting index.
    ///
    /// This private method is the core of BumpyVector. Given an arbitrary
    /// offset within the BumpyVector, determine which entry exists in it (even
    /// if the entry starts to the "left").
    ///
    /// The initial implementation is somewhat naive: loop from the
    /// `starting_index` to 0, searching for an object. If found, check the
    /// object's size to ensure it overlaps the `starting_index`.
    fn get_entry_start(&self, starting_index: usize) -> Option<usize> {
        // Get the meta-entry
        match self.data.get(&starting_index) {
            // If there's no meta-entry, nothing to do
            None => None,

            // If there IS a meta entry...
            Some(d) => {
                match d {
                    // Either we're exactly on the entry, and we can say so
                    MetaBumpyEntry::Something(_) => Some(starting_index),

                    // .. or we're just past it and can return the actual result
                    MetaBumpyEntry::NearlySomething(i) => Some(*i),
                }
            },
        }
    }

    /// Insert a new entry.
    ///
    /// # Return
    ///
    /// Returns `Ok(())` if successfully inserted. If it would overlap another
    /// entry or exceed `max_size`, return `Err(&str)` with a descriptive error
    /// string.
    ///
    /// Size must be at least 1.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::{BumpyEntry, BumpyVector};
    ///
    /// // Create a 10-byte `BumpyVector`
    /// let mut v: BumpyVector<&str> = BumpyVector::new(10);
    ///
    /// // Insert a 2-byte value starting at index 5 (using BumpyEntry directly)
    /// assert!(v.insert(BumpyEntry { entry: "hello", range: 5..7 }).is_ok());
    ///
    /// // Insert another 2-byte value starting at index 7 (using into())
    /// assert!(v.insert(("hello", 7..9).into()).is_ok());
    ///
    /// // Fail to insert a value that would overlap the first
    /// assert!(v.insert(("hello", 4..6).into()).is_err());
    ///
    /// // Fail to insert a value that would overlap the second
    /// assert!(v.insert(("hello", 6..7).into()).is_err());
    ///
    /// // Fail to insert a value that would go out of bounds
    /// assert!(v.insert(("hello", 100..1).into()).is_err());
    /// ```
    pub fn insert(&mut self, entry: BumpyEntry<T>) -> SimpleResult<()> {
        if entry.range.is_empty() {
            bail!("An entry can't have an empty range");
        }

        if entry.range.end > self.max_size {
            bail!("Invalid entry: entry exceeds max size (entry ends @ {:x}, max size is {:x})", entry.range.end, self.max_size);
        }

        // Check if there's a conflict on the left
        if self.data.get(&entry.range.start).is_some() {
            bail!("Invalid entry: overlaps another object");
        }

        // Check if there's a conflict on the right
        for x in entry.range.clone() {
            if self.data.contains_key(&x) {
                bail!("Invalid entry: overlaps another object");
            }
        }

        // Create meta-entries for the full range starting at the first
        for i in (entry.range.start + 1)..(entry.range.end) {
            self.data.insert(i, MetaBumpyEntry::NearlySomething(entry.range.start));
        }

        // Insert the actual entry
        self.data.insert(entry.range.start, MetaBumpyEntry::Something(entry));

        Ok(())
    }

    /// Insert an entry that implemented `AutoBumpyEntry`.
    ///
    /// The advantage of using this is that the entry knows its own size and
    /// index, so the insertion code is much cleaner.
    ///
    /// See `insert()` for return and errors.
    ///
    /// ```
    /// use libh2gb::bumpy_vector::{BumpyVector, AutoBumpyEntry};
    /// use std::ops::Range;
    ///
    /// // A struct that has an index and size, then whatever other data
    /// struct MyAutoEntry {
    ///     size: usize,
    ///     index: usize,
    ///     // ...anything else
    /// };
    ///
    /// // Simplest possible implementation for MyAutoEntry
    /// impl AutoBumpyEntry for MyAutoEntry {
    ///     fn range(&self) -> Range<usize> {
    ///         self.index..(self.index + self.size)
    ///     }
    /// }
    ///
    /// // Create an entry
    /// let entry = MyAutoEntry { index: 0, size: 10 };
    ///
    /// // Create a BumpyVector with the type we created
    /// let mut h: BumpyVector<MyAutoEntry> = BumpyVector::new(10);
    ///
    /// // Insert it
    /// assert!(h.insert_auto(entry).is_ok());
    /// ```
    pub fn insert_auto(&mut self, entry: T) -> SimpleResult<()>
    where T: AutoBumpyEntry {
        self.insert(entry.into())
    }

    /// Remove and return the entry at `index`.
    ///
    /// Note that the entry doesn't necessarily need to *start* at `index`,
    /// just overlap it.
    ///
    /// This does a lot of validation; if the array ever gets corrupted such
    /// that the start and middle entries are wrong, this is where it'll crash
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::BumpyVector;
    ///
    /// // Create a 10-byte `BumpyVector`
    /// let mut v: BumpyVector<&str> = BumpyVector::new(10);
    ///
    /// // Insert some data
    /// v.insert(("hello", 0..4).into()).unwrap();
    /// v.insert(("hello", 4..8).into()).unwrap();
    ///
    /// assert!(v.remove(0).is_some());
    /// assert!(v.remove(0).is_none());
    ///
    /// assert!(v.remove(6).is_some());
    /// assert!(v.remove(6).is_none());
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<BumpyEntry<T>> {
        // Try to get the real offset
        let real_offset = self.get_entry_start(index)?;

        // Get the entry from the starting location
        let actual_entry = match self.data.remove(&real_offset)? {
            MetaBumpyEntry::Something(e)       => e,
            MetaBumpyEntry::NearlySomething(_) => panic!("Something went wrong in our BumpyVector: found a middle entry where a starting entry belongs (at index {})", real_offset),
        };

        // Remove the NearlySomething entries associated with it
        for i in (actual_entry.range.start + 1)..(actual_entry.range.end) {
            match self.data.remove(&i) {
                None                                     => panic!("Something went wrong in our BumpyVector: missing an entry where there should be an entry (at index {}, starting range {:?})", i, actual_entry.range),
                Some(MetaBumpyEntry::Something(_))       => panic!("Something went wrong in our BumpyVector: found a starting entry where a middle entry belongs (at index {}, range {:?})", i, actual_entry.range),
                Some(MetaBumpyEntry::NearlySomething(_)) => (),
            }
        }

        Some(actual_entry)
    }

    /// Remove and return a range of entries.
    ///
    /// Ranges work as expected for Rust ranges; see `std::ops::Range`.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::BumpyVector;
    ///
    /// // Create a 10-byte `BumpyVector`
    /// let mut v: BumpyVector<&str> = BumpyVector::new(10);
    ///
    /// // Insert some data
    /// v.insert(("hello", 0..4).into()).unwrap();
    /// v.insert(("hello", 4..8).into()).unwrap();
    ///
    /// assert_eq!(2, v.remove_range(0..10).len());
    /// assert_eq!(0, v.remove_range(0..10).len());
    /// ```
    pub fn remove_range(&mut self, range: Range<usize>) -> Vec<BumpyEntry<T>> {
        let mut result: Vec<BumpyEntry<T>> = Vec::new();

        for i in range {
            if let Some(e) = self.remove(i) {
                result.push(e);
            }
        }

        result
    }

    /// Return a reference to an entry at the given index.
    ///
    /// Note that the entry doesn't necessarily need to *start* at the given
    /// index, it can simply be contained there.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::BumpyVector;
    ///
    /// // Create a 10-byte `BumpyVector`
    /// let mut v: BumpyVector<&str> = BumpyVector::new(10);
    ///
    /// // Insert some data
    /// v.insert(("hello", 0..4).into()).unwrap();
    ///
    /// assert!(v.get(0).is_some());
    /// assert!(v.get(1).is_some());
    /// assert!(v.get(2).is_some());
    /// assert!(v.get(3).is_some());
    /// assert!(v.get(4).is_none());
    /// assert!(v.get(5).is_none());
    ///
    /// assert_eq!("hello", v.get(0).unwrap().entry);
    /// assert_eq!("hello", v.get(1).unwrap().entry);
    /// assert_eq!("hello", v.get(2).unwrap().entry);
    /// assert_eq!("hello", v.get(3).unwrap().entry);
    /// ```
    pub fn get(&self, index: usize) -> Option<&BumpyEntry<T>> {
        // Try to get the real offset
        let real_offset = self.get_entry_start(index)?;

        // If there's no element, return none
        match self.data.get(&real_offset)? {
            MetaBumpyEntry::Something(e) => Some(e),
            MetaBumpyEntry::NearlySomething(_) => panic!("Something went wrong in BumpyVector: get_entry_start() returned a body entry instead of starting entry @ index {}", index),
        }
    }

    /// Return a mutable reference to an entry at the given index.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::BumpyVector;
    ///
    /// // Create a small BumpyVector
    /// let mut h: BumpyVector<String> = BumpyVector::new(10);
    ///
    /// // Insert a string to the start
    /// h.insert((String::from("hello"), 0..2).into()).unwrap();
    /// assert_eq!("hello", h.get(0).unwrap().entry);
    /// assert_eq!("hello", h.get(1).unwrap().entry);
    ///
    /// // Get a mutable reference to the string
    /// let s = h.get_mut(1).unwrap();
    ///
    /// // Modify it somehow
    /// s.entry.make_ascii_uppercase();
    ///
    /// // Verify it's changed
    /// assert_eq!("HELLO", h.get(0).unwrap().entry);
    /// assert_eq!("HELLO", h.get(1).unwrap().entry);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut BumpyEntry<T>> {
        // Try to get the real offset
        let real_offset = self.get_entry_start(index)?;

        // If there's no element, return none
        match self.data.get_mut(&real_offset)? {
            MetaBumpyEntry::Something(e) => Some(e),
            MetaBumpyEntry::NearlySomething(_) => panic!("Something went wrong in BumpyVector: get_entry_start() returned a body entry instead of starting entry @ index {}", index),
        }
    }

    /// Return a reference to an entry that *starts at* the given index.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::BumpyVector;
    ///
    /// // Create a 10-byte `BumpyVector`
    /// let mut v: BumpyVector<&str> = BumpyVector::new(10);
    ///
    /// // Insert some data
    /// v.insert(("hello", 0..4).into()).unwrap();
    ///
    /// assert!(v.get_exact(0).is_some());
    /// assert!(v.get_exact(1).is_none());
    /// assert!(v.get_exact(2).is_none());
    /// assert!(v.get_exact(3).is_none());
    /// assert!(v.get_exact(4).is_none());
    /// assert!(v.get_exact(5).is_none());
    ///
    /// assert_eq!("hello", v.get_exact(0).unwrap().entry);
    /// ```
    pub fn get_exact(&self, index: usize) -> Option<&BumpyEntry<T>> {
        // Just drop NearlySomething entries here
        match self.data.get(&index)? {
            MetaBumpyEntry::Something(e) => Some(e),
            MetaBumpyEntry::NearlySomething(_) => None,
        }
    }

    /// Return a mutable reference to an entry at exactly the given index.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::BumpyVector;
    ///
    /// // Create a small BumpyVector
    /// let mut h: BumpyVector<String> = BumpyVector::new(10);
    ///
    /// // Insert a string to the start
    /// h.insert((String::from("hello"), 0..2).into()).unwrap();
    /// assert_eq!("hello", h.get_exact(0).unwrap().entry);
    /// assert!(h.get_exact(1).is_none());
    ///
    /// // Get a mutable reference to the string
    /// let s = h.get_exact_mut(0).unwrap();
    ///
    /// // Modify it somehow
    /// s.entry.make_ascii_uppercase();
    ///
    /// // Verify it's changed
    /// assert_eq!("HELLO", h.get_exact(0).unwrap().entry);
    /// assert!(h.get_exact(1).is_none());
    /// ```
    pub fn get_exact_mut(&mut self, index: usize) -> Option<&mut BumpyEntry<T>> {
        // Just drop NearlySomething entries here
        match self.data.get_mut(&index)? {
            MetaBumpyEntry::Something(e) => Some(e),
            MetaBumpyEntry::NearlySomething(_) => None,
        }
    }

    /// Return a vector of entries within the given range.
    ///
    /// Note that the first entry doesn't need to *start* at the given start
    /// index it can simply be contained therein.
    ///
    /// Ranges work as expected for Rust ranges; see `std::ops::Range`.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::bumpy_vector::BumpyVector;
    ///
    /// // Create a 10-byte `BumpyVector`
    /// let mut v: BumpyVector<&str> = BumpyVector::new(10);
    ///
    /// // Insert some data with a gap in the middle
    /// v.insert(("hello", 0..2).into()).unwrap();
    /// v.insert(("hello", 4..6).into()).unwrap();
    ///
    /// assert_eq!(1, v.get_range(0..1).len());
    /// assert_eq!(1, v.get_range(0..2).len());
    /// assert_eq!(1, v.get_range(0..3).len());
    /// assert_eq!(1, v.get_range(0..4).len());
    /// assert_eq!(2, v.get_range(0..5).len());
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if an entry's size is 0. That shouldn't be possible short of
    /// tinkering with internal state.
    pub fn get_range(&self, range: Range<usize>) -> Vec<&BumpyEntry<T>> {
        // We're stuffing all of our data into a vector to iterate over it
        let mut result: Vec<&BumpyEntry<T>> = Vec::new();

        // Start at the first entry left of what they wanted, if it exists
        let mut i = match self.get_entry_start(range.start) {
            Some(e) => e,
            None    => range.start,
        };

        // From here on, `i` should never point at a "body" entry
        while i < range.end && i < self.max_size {
            match self.data.get(&i) {
                None => {
                    i += 1;
                },
                Some(MetaBumpyEntry::Something(e)) => {
                    if e.range.is_empty() {
                        panic!("Something went wrong in BumpyVector: an entry in get_range had an empty range, which is bad news (index {}, overall range {:?})", i, range);
                    }

                    result.push(e);
                    i = e.range.end;
                },
                Some(MetaBumpyEntry::NearlySomething(_)) => {
                    panic!("Ran into a middle entry where a start entry expected in get_range (index {}, range {:?})", i, range);
                },
            }
        }

        result
    }

    /// Returns the number of entries.
    ///
    /// This isn't the most efficient operation, unfortunately; we can probably
    /// improve if we need to
    pub fn len(&self) -> usize {
        self.get_range(0..self.max_size()).len()
    }

    pub fn max_size(&self) -> usize {
        return self.max_size;
    }
}

/// Convert into an iterator.
///
/// Naively iterate across all entries, move them into a `Vec<_>`, and convert
/// that vector into an iterator.
///
impl<'a, T> IntoIterator for &'a BumpyVector<T> {
    type Item = &'a BumpyEntry<T>;
    type IntoIter = std::vec::IntoIter<&'a BumpyEntry<T>>;

    fn into_iter(self) -> std::vec::IntoIter<&'a BumpyEntry<T>> {
        return self.get_range(0..self.max_size).into_iter();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_insert() {
        let mut h: BumpyVector<&str> = BumpyVector::new(100);

        // Insert a 5-byte value at 10
        h.insert(("hello", 10..15).into()).unwrap();
        assert_eq!(1, h.len());

        // Earlier values are none
        assert!(h.get(8).is_none());
        assert!(h.get(9).is_none());

        // Middle values are all identical, no matter where in the entry we
        // retrieve it
        assert_eq!("hello", h.get(10).unwrap().entry);
        assert_eq!(10..15,  h.get(10).unwrap().range);

        assert_eq!("hello", h.get(11).unwrap().entry);
        assert_eq!(10..15,  h.get(11).unwrap().range);

        assert_eq!("hello", h.get(12).unwrap().entry);
        assert_eq!(10..15,  h.get(12).unwrap().range);

        assert_eq!("hello", h.get(13).unwrap().entry);
        assert_eq!(10..15,  h.get(13).unwrap().range);

        assert_eq!("hello", h.get(14).unwrap().entry);
        assert_eq!(10..15,  h.get(14).unwrap().range);

        // Last couple entries are none
        assert!(h.get(15).is_none());
        assert!(h.get(16).is_none());

        // There should still be a single entry
        assert_eq!(1, h.len());
    }

    #[test]
    fn test_zero_sized_insert() {
        let mut h: BumpyVector<&str> = BumpyVector::new(100);

        // Insert a 0-byte array
        assert!(h.insert(("hello", 10..10).into()).is_err());
        assert_eq!(0, h.len());
    }

    #[test]
    fn test_overlapping_one_byte_inserts() {
        let mut h: BumpyVector<&str> = BumpyVector::new(100);

        // Insert a 2-byte value at 10
        h.insert(("hello", 10..12).into()).unwrap();
        assert_eq!(1, h.len());

        // We can insert before
        assert!(h.insert(("ok", 8..9).into()).is_ok());
        assert_eq!(2, h.len());
        assert!(h.insert(("ok", 9..10).into()).is_ok());
        assert_eq!(3, h.len());

        // We can't insert within
        assert!(h.insert(("error", 10..11).into()).is_err());
        assert!(h.insert(("error", 11..12).into()).is_err());
        assert_eq!(3, h.len());

        // We can insert after
        assert!(h.insert(("ok", 12..13).into()).is_ok());
        assert_eq!(4, h.len());
        assert!(h.insert(("ok", 13..14).into()).is_ok());
        assert_eq!(5, h.len());
    }

    #[test]
    fn test_overlapping_multi_byte_inserts() {
        // Define 10-12, put something at 7-9 (good!)
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(("hello", 10..13).into()).unwrap();
        assert!(h.insert(("ok", 7..10).into()).is_ok());

        // Define 10-12, try every overlapping bit
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(BumpyEntry::from(("hello", 10..13))).unwrap();
        assert!(h.insert(("error", 8..11).into()).is_err());
        assert!(h.insert(("error", 9..12).into()).is_err());
        assert!(h.insert(("error", 10..13).into()).is_err());
        assert!(h.insert(("error", 11..14).into()).is_err());
        assert!(h.insert(("error", 12..15).into()).is_err());

        // 6-9 and 13-15 will work
        assert!(h.insert(BumpyEntry::from(("ok", 6..9))).is_ok());
        assert!(h.insert(("ok", 13..16).into()).is_ok());
        assert_eq!(3, h.len());
    }

    #[test]
    fn test_remove() {
        // Define 10-12, put something at 7-9 (good!)
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(("hello", 8..10).into()).unwrap();
        h.insert(("hello", 10..12).into()).unwrap();
        h.insert(("hello", 12..14).into()).unwrap();
        assert_eq!(3, h.len());

        // Remove from the start of an entry
        let e = h.remove(10).unwrap();
        assert_eq!("hello", e.entry);
        assert_eq!(10..12,  e.range);
        assert_eq!(2,       h.len());
        assert!(h.get(10).is_none());
        assert!(h.get(11).is_none());

        // Put it back
        h.insert(("hello", 10..12).into()).unwrap();
        assert_eq!(3, h.len());

        // Remove from the middle of an entry
        let e = h.remove(11).unwrap();
        assert_eq!("hello", e.entry);
        assert_eq!(10..12,  e.range);
        assert_eq!(2,       h.len());
        assert!(h.get(10).is_none());
        assert!(h.get(11).is_none());

        // Remove 11 again, which is nothing
        let result = h.remove(11);
        assert!(result.is_none());

        let e = h.remove(13).unwrap();
        assert_eq!("hello", e.entry);
        assert_eq!(12..14,  e.range);
        assert_eq!(1,       h.len());
        assert!(h.get(12).is_none());
        assert!(h.get(13).is_none());

        h.remove(8);
        assert_eq!(0, h.len());
        assert!(h.get(8).is_none());
        assert!(h.get(9).is_none());
    }

    #[test]
    fn test_beginning() {
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert(("hello", 0..2).into()).unwrap();

        assert_eq!(1, h.len());

        assert_eq!("hello", h.get(0).unwrap().entry);
        assert_eq!(0..2,    h.get(0).unwrap().range);

        assert_eq!("hello", h.get(1).unwrap().entry);
        assert_eq!(0..2,    h.get(1).unwrap().range);

        assert!(h.get(2).is_none());
    }

    #[test]
    fn test_max_size() {
        // Inserting at 7-8-9 works
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert_eq!(10, h.max_size());

        h.insert(("hello", 7..10).into()).unwrap();
        assert_eq!(1, h.len());

        // Inserting at 8-9-10 and onward does not
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert(("hello", 8..11).into()).is_err());
        assert_eq!(0, h.len());

        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert(("hello", 9..12).into()).is_err());
        assert_eq!(0, h.len());

        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert(("hello", 10..13).into()).is_err());
        assert_eq!(0, h.len());

        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert(("hello", 11..14).into()).is_err());
        assert_eq!(0, h.len());
    }

    #[test]
    fn test_remove_range() {
        // Create an object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(("hello", 8..10).into()).unwrap();
        h.insert(("hello", 10..12).into()).unwrap();
        h.insert(("hello", 12..14).into()).unwrap();
        assert_eq!(3, h.len());

        // Test removing the first two entries
        let result = h.remove_range(8..12);
        assert_eq!(1, h.len());
        assert_eq!(2, result.len());

        assert_eq!("hello", result[0].entry);
        assert_eq!(8..10,   result[0].range);

        assert_eq!("hello", result[1].entry);
        assert_eq!(10..12,  result[1].range);

        // Re-create the object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(("hello", 8..10).into()).unwrap();
        h.insert(("hello", 10..12).into()).unwrap();
        h.insert(("hello", 12..14).into()).unwrap();
        assert_eq!(3, h.len());

        // Test where the first entry starts left of the actual starting index
        let result = h.remove_range(9..11);
        assert_eq!(1, h.len());
        assert_eq!(2, result.len());

        assert_eq!("hello", result[0].entry);
        assert_eq!(8..10,   result[0].range);

        assert_eq!("hello", result[1].entry);
        assert_eq!(10..12,  result[1].range);

        // Re-create the object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(("hello", 8..10).into()).unwrap();
        h.insert(("hello", 10..12).into()).unwrap();
        h.insert(("hello", 12..14).into()).unwrap();
        assert_eq!(3, h.len());

        // Test the entire object
        let result = h.remove_range(0..1000);
        assert_eq!(0, h.len());
        assert_eq!(3, result.len());

        assert_eq!("hello", result[0].entry);
        assert_eq!(8..10,   result[0].range);

        assert_eq!("hello", result[1].entry);
        assert_eq!(10..12,  result[1].range);
    }

    #[test]
    fn test_get() {
        // Create an object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(("hello", 8..10).into()).unwrap();

        // Test removing the first two entries
        assert!(h.get(7).is_none());
        assert!(h.get(8).is_some());
        assert!(h.get(9).is_some());
        assert!(h.get(10).is_none());
    }

    #[test]
    fn test_get_mut() {
        // Create an object
        let mut h: BumpyVector<String> = BumpyVector::new(100);
        h.insert((String::from("hello"), 8..10).into()).unwrap();

        // Get a mutable reference
        let s = h.get_mut(9).unwrap();
        s.entry.make_ascii_uppercase();

        let s2 = h.get(8).unwrap();
        assert_eq!("HELLO", s2.entry);
    }

    #[test]
    fn test_get_exact() {
        // Create an object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert(("hello", 8..10).into()).unwrap();

        // Test removing the first two entries
        assert!(h.get_exact(7).is_none());
        assert!(h.get_exact(8).is_some());
        assert!(h.get_exact(9).is_none());
        assert!(h.get_exact(10).is_none());
    }

    #[test]
    fn test_get_exact_mut() {
        // Create an object
        let mut h: BumpyVector<String> = BumpyVector::new(100);
        h.insert((String::from("hello"), 8..10).into()).unwrap();

        // Make sure it's actually exist
        assert!(h.get_exact_mut(9).is_none());

        // Get a mutable reference
        let s = h.get_exact_mut(8).unwrap();
        s.entry.make_ascii_uppercase();

        let s = h.get_exact(8).unwrap();
        assert_eq!("HELLO", s.entry);
    }

    #[test]
    fn test_get_range() {
        // Create a BumpyVector that looks like:
        //
        // [--0-- --1-- --2-- --3-- --4-- --5-- --6-- --7-- --8-- --9--]
        //        +-----------------            +----------------+
        //        |   "a" (2)| "b" |            |      "c"       |
        //        +----------+------            +----------------+
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert(("a", 1..3).into()).unwrap();
        h.insert(("b", 3..4).into()).unwrap();
        h.insert(("c", 6..9).into()).unwrap();

        // Get just the first two
        let result = h.get_range(2..6);
        assert_eq!(2, result.len());

        // Get the first two, then just barely the third
        let result = h.get_range(2..7);
        assert_eq!(3, result.len());

        // Get the first two again, starting further left
        let result = h.get_range(1..6);
        assert_eq!(2, result.len());

        // Get all three again
        let result = h.get_range(1..7);
        assert_eq!(3, result.len());

        // Get way more than everything
        let result = h.get_range(0..100);
        assert_eq!(3, result.len());
    }

    #[test]
    fn test_iterator() {
        // Create a BumpyVector that looks like:
        //
        // [--0-- --1-- --2-- --3-- --4-- --5-- --6-- --7-- --8-- --9--]
        //        +-----------------            +----------------+
        //        |   "a" (2)| "b" |            |      "c"       |
        //        +----------+------            +----------------+
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert(("a", 1..3).into()).unwrap();
        h.insert(("b", 3..4).into()).unwrap();
        h.insert(("c", 6..9).into()).unwrap();

        let mut iter = h.into_iter();

        // Entry "a" (index 1-2)
        let e = iter.next().unwrap();
        assert_eq!("a",  e.entry);
        assert_eq!(1..3, e.range);

        // Entry "b" (index 3)
        let e = iter.next().unwrap();
        assert_eq!("b",  e.entry);
        assert_eq!(3..4, e.range);

        // Entry "c" (index 6-8)
        let e = iter.next().unwrap();
        assert_eq!("c",  e.entry);
        assert_eq!(6..9, e.range);

        // That's it!
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_serialize() {
        let mut h: BumpyVector<String> = BumpyVector::new(10);
        h.insert((String::from("a"), 1..3).into()).unwrap();
        h.insert((String::from("b"), 3..4).into()).unwrap();
        h.insert((String::from("c"), 6..9).into()).unwrap();

        // Serialize
        let serialized = ron::ser::to_string(&h).unwrap();

        // Deserialize
        let h: BumpyVector<String> = ron::de::from_str(&serialized).unwrap();

        // Make sure we have the same entries
        assert_eq!("a",  h.get(2).unwrap().entry);
        assert_eq!(1..3, h.get(2).unwrap().range);
        assert_eq!("b",  h.get(3).unwrap().entry);
        assert!(h.get(4).is_none());
        assert!(h.get(5).is_none());
        assert_eq!("c",  h.get(6).unwrap().entry);
        assert_eq!(6..9, h.get(6).unwrap().range);
    }

    #[test]
    fn test_clone() {
        let mut h: BumpyVector<String> = BumpyVector::new(10);
        h.insert((String::from("a"), 1..3).into()).unwrap();
        h.insert((String::from("b"), 3..4).into()).unwrap();
        h.insert((String::from("c"), 6..9).into()).unwrap();

        // Serialize
        let cloned = h.clone();

        // Make sure we have the same entries
        assert_eq!("a",  cloned.get(2).unwrap().entry);
        assert_eq!(1..3, cloned.get(2).unwrap().range);
        assert_eq!("b",  cloned.get(3).unwrap().entry);
        assert!(cloned.get(4).is_none());
        assert!(cloned.get(5).is_none());
        assert_eq!("c",  cloned.get(6).unwrap().entry);
        assert_eq!(6..9, cloned.get(6).unwrap().range);
    }

    #[test]
    fn test_auto_bumpy_entry() {
        struct Test {
            size: usize,
            index: usize,
        }

        impl AutoBumpyEntry for Test {
            fn range(&self) -> Range<usize> {
                self.index..(self.index+self.size)
            }
        }

        let good_entry1 = Test {
            index: 0,
            size: 10,
        };

        let good_entry2 = Test {
            index: 10,
            size: 10,
        };

        let bad_entry = Test {
            index: 5,
            size: 10,
        };

        let mut h: BumpyVector<Test> = BumpyVector::new(100);

        // Insert a 5-byte value at 10
        assert!(h.insert(good_entry1.into()).is_ok());
        assert_eq!(1, h.len());

        assert!(h.insert(good_entry2.into()).is_ok());
        assert_eq!(2, h.len());

        assert!(h.insert(bad_entry.into()).is_err());
        assert_eq!(2, h.len());
    }
}
