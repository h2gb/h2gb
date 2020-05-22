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

impl<T> BumpyVector<T> {
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

    // pub fn remove_range(&mut self, index: usize, length: usize) -> Vec<t> {
    //     // TODO
    // }

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

    // TODO(ron): Can this be used for into_iterator?
    // pub fn get_range(&self, index: usize, length: usize) -> Vec<(Option<T>, usize, usize)> {
    // TODO
    // }

    pub fn len(&self) -> usize {
        return self.data.len();
    }
}

impl<'a, T> IntoIterator for &'a BumpyVector<T> {
    type Item = (Option<&'a T>, usize, usize);
    type IntoIter = std::vec::IntoIter<(Option<&'a T>, usize, usize)>;

    fn into_iter(self) -> std::vec::IntoIter<(Option<&'a T>, usize, usize)> {
        // We're stuffing all of our data into a vector to iterate over it
        let mut real_data: Vec<(Option<&'a T>, usize, usize)> = Default::default();

        // Loop through by counting, since each entry takes up multiple indices
        let mut a = 0;
        while a < self.max_size {
            // Pull the entry out, if it exists
            if let Some(e) = self.data.get(&a) {
                // Add the entry to the vector, and jump over it
                real_data.push((Some(&e.entry), a, e.size));
                a += e.size;
            } else {
                // If the user wants empty elements, push a fake entry
                if self.iterate_over_empty {
                    real_data.push((None, a, 1));
                }
                a += 1;
            }
        }

        // Convert the vector into an iterator
        return real_data.into_iter();
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

        // TODO: Check return, remove from middle
        h.remove(10);
        assert_eq!(h.len(), 2);
        assert_eq!(h.get(10), None);
        assert_eq!(h.get(11), None);

        h.insert("hello", 10, 2).unwrap();
        assert_eq!(h.len(), 3);

        h.remove(11);
        assert_eq!(h.len(), 2);
        assert_eq!(h.get(10), None);
        assert_eq!(h.get(11), None);

        h.remove(13);
        assert_eq!(h.len(), 1);
        assert_eq!(h.get(12), None);
        assert_eq!(h.get(13), None);

        h.remove(8);
        assert_eq!(h.len(), 0);
        assert_eq!(h.get(8), None);
        assert_eq!(h.get(9), None);
    }

    #[test]
    fn beginning_works() {
        let mut h: BumpyVector<&str> = BumpyVector::new(10);
        h.insert("hello", 0, 2).unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h.get(0).unwrap(), (&"hello", 0, 2));
        assert_eq!(h.get(1).unwrap(), (&"hello", 0, 2));
        assert_eq!(h.get(2), None);
    }

    #[test]
    fn max_size() {
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
}
