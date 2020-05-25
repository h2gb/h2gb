//! Provides a vector-like object where elements can be larger than one "space"

#![allow(dead_code)]

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BumpyEntry<T> {
    entry: T,
    size: usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BumpyVector<T> {
    data: HashMap<usize, BumpyEntry<T>>,
    max_size: usize,
    iterate_over_empty: bool,
}

impl<'a, T> BumpyVector<T> {
    fn new(max_size: usize) -> Self {
        BumpyVector {
            data: HashMap::new(),
            max_size: max_size,
            iterate_over_empty: false,
        }
    }

    fn find_left_offset(&self, starting_index: usize) -> Option<usize> {
        // Keep a handle to the starting index
        let mut index = starting_index;

        // Loop right to zero
        loop {
            // Check if we have data at the index
            match self.data.get(&index) {
                // If there's a value, we're set!
                Some(d) => {
                    // If we were too far away, it doesn't count. No value!
                    if d.size <= (starting_index - index) {
                        return None;
                    }

                    // Otherwise, we have the real index!
                    return Some(index);
                },

                // If there's no value, we keep going
                None => {
                    if index == 0 {
                        return None;
                    }

                    index -= 1;
                },
            };
        }
    }

    pub fn insert(&mut self, value: T, index: usize, size: usize) -> Result<(), &'static str> {
        if index + size > self.max_size {
            return Err("Invalid entry: entry exceeds max size");
        }

        // Check if there's a conflict on the left
        if self.find_left_offset(index).is_some() {
            return Err("Uh oh!");
        }

        // Check if there's a conflict on the right
        for x in index..(index + size) {
            if self.data.contains_key(&x) {
                return Err("Uh oh!");
            }
        }

        // We're good, so create an entry!
        self.data.insert(index, BumpyEntry {
            entry: value,
            size: size,
        });

        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Option<(T, usize, usize)> {
        // Try to get the real offset
        let real_offset = self.find_left_offset(index);

        // If there's no element, return none
        if let Some(o) = real_offset {
            // Remove it!
            if let Some(d) = self.data.remove(&o) {
                return Some((d.entry, o, d.size));
            }
        }

        None
    }

    pub fn remove_range(&mut self, index: usize, length: usize) -> Vec<(T, usize, usize)> {
        let mut result: Vec<(T, usize, usize)> = Vec::new();

        for i in index..(index+length) {
            if let Some(e) = self.remove(i) {
                result.push(e);
            }
        }

        result
    }

    // Returns a tuple of: a reference to the entry, the starting address, and the size
    pub fn get(&self, index: usize) -> Option<(&T, usize, usize)> {
        // Try to get the real offset
        let real_offset = self.find_left_offset(index);

        // If there's no element, return none
        if let Some(o) = real_offset {
            // Get the entry itself from the address
            let entry = self.data.get(&o);

            // Although this probably won't fail, we need to check!
            if let Some(e) = entry {
                // Return the entry
                return Some((&e.entry, o, e.size));
            }
        }

        None
    }

    // Return an entry if it starts at the exact address
    pub fn get_exact(&self, index: usize) -> Option<(&T, usize, usize)> {
        match self.data.get(&index) {
            Some(o) => Some((&o.entry, index, o.size)),
            None    => None,
        }
    }

    pub fn get_range(&self, start: usize, length: usize, include_empty: bool) -> Vec<(Option<&T>, usize, usize)> {
        // We're stuffing all of our data into a vector to iterate over it
        let mut result: Vec<(Option<&T>, usize, usize)> = Vec::new();

        // Start at the first entry left of what they wanted, if it exists
        let mut i = match self.find_left_offset(start) {
            Some(e) => e,
            None    => start,
        };

        // Loop up to <length> bytes after the starting index
        while i < start + length && i < self.max_size {
            // Pull the entry out, if it exists
            if let Some(e) = self.data.get(&i) {
                // Add the entry to the vector, and jump over it
                result.push((Some(&e.entry), i, e.size));
                i += e.size;
            } else {
                // If the user wants empty elements, push i fake entry
                if include_empty {
                    result.push((None, i, 1));
                }
                i += 1;
            }
        }

        result
    }

    pub fn len(&self) -> usize {
        // Return the number of entries
        return self.data.len();
    }
}

impl<'a, T> IntoIterator for &'a BumpyVector<T> {
    type Item = (Option<&'a T>, usize, usize);
    type IntoIter = std::vec::IntoIter<(Option<&'a T>, usize, usize)>;

    fn into_iter(self) -> std::vec::IntoIter<(Option<&'a T>, usize, usize)> {
        return self.get_range(0, self.max_size, self.iterate_over_empty).into_iter();
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
        h.insert("hello", 10, 5).unwrap();
        assert_eq!(h.len(), 1);

        // Make sure only those 5 values are defined
        assert_eq!(h.get(8), None);
        assert_eq!(h.get(9), None);
        assert_eq!(h.get(10).unwrap(), (&"hello", 10, 5));
        assert_eq!(h.get(11).unwrap(), (&"hello", 10, 5));
        assert_eq!(h.get(12).unwrap(), (&"hello", 10, 5));
        assert_eq!(h.get(13).unwrap(), (&"hello", 10, 5));
        assert_eq!(h.get(14).unwrap(), (&"hello", 10, 5));
        assert_eq!(h.get(15), None);
        assert_eq!(h.get(16), None);
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn test_overlapping_one_byte_inserts() {
        let mut h: BumpyVector<&str> = BumpyVector::new(100);

        // Insert a 2-byte value at 10
        h.insert("hello", 10, 2).unwrap();
        assert_eq!(h.len(), 1);

        // We can insert before
        assert!(h.insert("ok", 8,  1).is_ok());
        assert_eq!(h.len(), 2);
        assert!(h.insert("ok", 9,  1).is_ok());
        assert_eq!(h.len(), 3);

        // We can't insert within
        assert!(h.insert("error", 10, 1).is_err());
        assert!(h.insert("error", 11, 1).is_err());
        assert_eq!(h.len(), 3);

        // We can insert after
        assert!(h.insert("ok", 12, 1).is_ok());
        assert_eq!(h.len(), 4);
        assert!(h.insert("ok", 13, 1).is_ok());
        assert_eq!(h.len(), 5);
    }

    #[test]
    fn test_overlapping_multi_byte_inserts() {
        // Define 10-12, put something at 7-9 (good!)
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 10, 3).unwrap();
        assert!(h.insert("ok", 7,  3).is_ok());

        // Define 10-12, try every overlapping bit
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 10, 3).unwrap();
        assert!(h.insert("error", 8,  3).is_err());
        assert!(h.insert("error", 9,  3).is_err());
        assert!(h.insert("error", 10, 3).is_err());
        assert!(h.insert("error", 11, 3).is_err());
        assert!(h.insert("error", 12, 3).is_err());

        // 6-9 and 13-15 will work
        assert!(h.insert("ok", 6,  3).is_ok());
        assert!(h.insert("ok", 13, 3).is_ok());
        assert_eq!(h.len(), 3);
    }

    #[test]
    fn test_remove() {
        // Define 10-12, put something at 7-9 (good!)
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 8, 2).unwrap();
        h.insert("hello", 10, 2).unwrap();
        h.insert("hello", 12, 2).unwrap();
        assert_eq!(h.len(), 3);

        // Remove from the start of an entry
        let (e, index, size) = h.remove(10).unwrap();
        assert_eq!(e, "hello");
        assert_eq!(index, 10);
        assert_eq!(size, 2);
        assert_eq!(h.len(), 2);
        assert_eq!(h.get(10), None);
        assert_eq!(h.get(11), None);

        // Put it back
        h.insert("hello", 10, 2).unwrap();
        assert_eq!(h.len(), 3);

        // Remove from the middle of an entry
        let (e, index, size) = h.remove(11).unwrap();
        assert_eq!(e, "hello");
        assert_eq!(index, 10);
        assert_eq!(size, 2);
        assert_eq!(h.len(), 2);
        assert_eq!(h.get(10), None);
        assert_eq!(h.get(11), None);

        // Remove 11 again, which is nothing
        let result = h.remove(11);
        assert_eq!(None, result);

        let (e, index, size) = h.remove(13).unwrap();
        assert_eq!(e, "hello");
        assert_eq!(index, 12);
        assert_eq!(size, 2);
        assert_eq!(h.len(), 1);
        assert_eq!(h.get(12), None);
        assert_eq!(h.get(13), None);

        h.remove(8);
        assert_eq!(h.len(), 0);
        assert_eq!(h.get(8), None);
        assert_eq!(h.get(9), None);
    }

    #[test]
    fn test_beginning() {
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert("hello", 0, 2).unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h.get(0).unwrap(), (&"hello", 0, 2));
        assert_eq!(h.get(1).unwrap(), (&"hello", 0, 2));
        assert_eq!(h.get(2), None);
    }

    #[test]
    fn test_max_size() {
        // Inserting at 7-8-9 works
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert("hello", 7, 3).unwrap();
        assert_eq!(h.len(), 1);

        // Inserting at 8-9-10 and onward does not
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert("hello", 8, 3).is_err());
        assert_eq!(h.len(), 0);

        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert("hello", 9, 3).is_err());
        assert_eq!(h.len(), 0);

        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert("hello", 10, 3).is_err());
        assert_eq!(h.len(), 0);

        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        assert!(h.insert("hello", 11, 3).is_err());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn test_remove_range() {
        // Create an object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 8, 2).unwrap();
        h.insert("hello", 10, 2).unwrap();
        h.insert("hello", 12, 2).unwrap();
        assert_eq!(h.len(), 3);

        // Test removing the first two entries
        let result = h.remove_range(8, 4);
        assert_eq!(h.len(), 1);
        assert_eq!(result.len(), 2);

        let (e, index, size) = result[0];
        assert_eq!(e, "hello");
        assert_eq!(index, 8);
        assert_eq!(size, 2);

        let (e, index, size) = result[1];
        assert_eq!(e, "hello");
        assert_eq!(index, 10);
        assert_eq!(size, 2);

        // Re-create the object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 8, 2).unwrap();
        h.insert("hello", 10, 2).unwrap();
        h.insert("hello", 12, 2).unwrap();
        assert_eq!(h.len(), 3);

        // Test where the first entry starts left of the actual starting index
        let result = h.remove_range(9, 2);
        assert_eq!(h.len(), 1);
        assert_eq!(result.len(), 2);

        let (e, index, size) = result[0];
        assert_eq!(e, "hello");
        assert_eq!(index, 8);
        assert_eq!(size, 2);

        let (e, index, size) = result[1];
        assert_eq!(e, "hello");
        assert_eq!(index, 10);
        assert_eq!(size, 2);

        // Re-create the object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 8, 2).unwrap();
        h.insert("hello", 10, 2).unwrap();
        h.insert("hello", 12, 2).unwrap();
        assert_eq!(h.len(), 3);

        // Test the entire object
        let result = h.remove_range(0, 1000);
        assert_eq!(h.len(), 0);
        assert_eq!(result.len(), 3);

        let (e, index, size) = result[0];
        assert_eq!(e, "hello");
        assert_eq!(index, 8);
        assert_eq!(size, 2);

        let (e, index, size) = result[1];
        assert_eq!(e, "hello");
        assert_eq!(index, 10);
        assert_eq!(size, 2);
    }

    #[test]
    fn test_get() {
        // Create an object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 8, 2).unwrap();

        // Test removing the first two entries
        assert_eq!(None, h.get(7));
        assert_ne!(None, h.get(8));
        assert_ne!(None, h.get(9));
        assert_eq!(None, h.get(10));
    }

    #[test]
    fn test_get_exact() {
        // Create an object
        let mut h: BumpyVector<&str> = BumpyVector::new(100);
        h.insert("hello", 8, 2).unwrap();

        // Test removing the first two entries
        assert_eq!(None, h.get_exact(7));
        assert_ne!(None, h.get_exact(8));
        assert_eq!(None, h.get_exact(9));
        assert_eq!(None, h.get_exact(10));
    }

    #[test]
    fn test_get_range_skip_empty() {
        // Create a BumpyVector that looks like:
        //
        // [--0-- --1-- --2-- --3-- --4-- --5-- --6-- --7-- --8-- --9--]
        //        +-----------------            +----------------+
        //        |   "a" (2)| "b" |            |      "c"       |
        //        +----------+------            +----------------+
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert("a", 1, 2).unwrap();
        h.insert("b", 3, 1).unwrap();
        h.insert("c", 6, 3).unwrap();

        // Get just the first two
        let result = h.get_range(2, 4, false);
        assert_eq!(result.len(), 2);

        // Get the first two, then just barely the third
        let result = h.get_range(2, 5, false);
        assert_eq!(result.len(), 3);

        // Get the first two again, starting further left
        let result = h.get_range(1, 5, false);
        assert_eq!(result.len(), 2);

        // Get all three again
        let result = h.get_range(1, 6, false);
        assert_eq!(result.len(), 3);

        // Get way more than everything
        let result = h.get_range(0, 100, false);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_get_range_include_empty() {
        // Create a BumpyVector that looks like:
        //
        // [--0-- --1-- --2-- --3-- --4-- --5-- --6-- --7-- --8-- --9--]
        //        +-----------------            +----------------+
        //        |   "a" (2)| "b" |            |      "c"       |
        //        +----------+------            +----------------+
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert("a", 1, 2).unwrap();
        h.insert("b", 3, 1).unwrap();
        h.insert("c", 6, 3).unwrap();

        // Get just the first two, plus two empty spots
        let result = h.get_range(2, 4, true);
        assert_eq!(result.len(), 4);

        // Get the first two, the two empty spots, then just barely the third
        let result = h.get_range(2, 5, true);
        assert_eq!(result.len(), 5);

        // Get an empty spot, then the first one
        let result = h.get_range(0, 3, true);
        assert_eq!(result.len(), 2);

        // Get an empty spot, then the first two
        let result = h.get_range(0, 4, true);
        assert_eq!(result.len(), 3);

        // Get the last one, then the empty spot after it, then we're at the end and should stop
        let result = h.get_range(8, 1000, true);
        assert_eq!(result.len(), 2);
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
        h.insert("a", 1, 2).unwrap();
        h.insert("b", 3, 1).unwrap();
        h.insert("c", 6, 3).unwrap();

        // Iterate over everything, including empty values
        h.iterate_over_empty = true;
        let mut iter = h.into_iter();
        assert_eq!(iter.next().unwrap(), (None, 0, 1));
        assert_eq!(iter.next().unwrap(), (Some(&"a"), 1, 2));
        assert_eq!(iter.next().unwrap(), (Some(&"b"), 3, 1));
        assert_eq!(iter.next().unwrap(), (None, 4, 1));
        assert_eq!(iter.next().unwrap(), (None, 5, 1));
        assert_eq!(iter.next().unwrap(), (Some(&"c"), 6, 3));
        assert_eq!(iter.next().unwrap(), (None, 9, 1));
        assert_eq!(iter.next(), None);

        // Using the same hashmap, this time skip empty values
        h.iterate_over_empty = false;
        let mut iter = h.into_iter();
        assert_eq!(iter.next().unwrap(), (Some(&"a"), 1, 2));
        assert_eq!(iter.next().unwrap(), (Some(&"b"), 3, 1));
        assert_eq!(iter.next().unwrap(), (Some(&"c"), 6, 3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_serialize() {
        let mut h: BumpyVector<String> = BumpyVector::new(10);
        h.insert(String::from("a"), 1, 2).unwrap();
        h.insert(String::from("b"), 3, 1).unwrap();
        h.insert(String::from("c"), 6, 3).unwrap();

        // Serialize
        let serialized = ron::ser::to_string(&h).unwrap();

        // Deserialize
        let h: BumpyVector<String> = ron::de::from_str(&serialized).unwrap();

        // Make sure we have the same entries
        assert_eq!(h.get(2).unwrap().0, "a");
        assert_eq!(h.get(2).unwrap().1, 1);
        assert_eq!(h.get(2).unwrap().2, 2);
        assert_eq!(h.get(3).unwrap().0, "b");
        assert_eq!(None, h.get(4));
        assert_eq!(None, h.get(5));
        assert_eq!(h.get(6).unwrap().0, "c");
    }
}
