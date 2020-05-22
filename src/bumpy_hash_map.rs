#![allow(dead_code)]

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BumpyEntry<T> {
    entry: T,
    size: usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
// TODO(ron) Rename to BumpyVector
pub struct BumpyHashMap<V> {
    data: HashMap<usize, BumpyEntry<V>>,
    max_size: usize,
    iterate_over_empty: bool,
}

impl<V> BumpyHashMap<V> {
    fn new(max_size: usize) -> Self {
        BumpyHashMap {
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

    // TODO(ron) Make the index/size/value trio more consistent
    pub fn insert(&mut self, index: usize, size: usize, value: V) -> Result<(), &'static str> {
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

    pub fn remove(&mut self, index: usize) {
        // Try to get the real offset
        let real_offset = self.find_left_offset(index);

        // If there's no element, return none
        if let Some(o) = real_offset {
            // Remove it!
            self.data.remove(&o);
        }
    }

    // Returns a tuple of: a reference to the entry, the starting address, and the size
    pub fn get(&self, index: usize) -> Option<(&V, usize, usize)> {
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

    pub fn entries(&self) -> usize {
        return self.data.len();
    }
}

impl<'a, V> IntoIterator for &'a BumpyHashMap<V> {
    type Item = (Option<&'a V>, usize, usize);
    type IntoIter = std::vec::IntoIter<(Option<&'a V>, usize, usize)>;

    fn into_iter(self) -> std::vec::IntoIter<(Option<&'a V>, usize, usize)> {
        let mut a = 0;
        let mut real_data: Vec<(Option<&'a V>, usize, usize)> = Default::default();
        while a < self.max_size {
            if self.data.contains_key(&a) {
                let entry = &self.data.get(&a);

                if let Some(e) = entry {
                    real_data.push((Some(&e.entry), a, e.size));
                    a += e.size;
                } else {
                    // This block shouldn't be able to happen
                    if self.iterate_over_empty {
                        real_data.push((None, a, 1));
                    }
                    a += 1;
                }
            } else {
                if self.iterate_over_empty {
                    real_data.push((None, a, 1));
                }
                a += 1;
            }
        }

        return real_data.into_iter();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_insert() {
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(100);

        // Insert a 5-byte value at 10
        h.insert(10, 5, "hello").unwrap();
        assert_eq!(h.entries(), 1);

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
        assert_eq!(h.entries(), 1);
    }

    #[test]
    fn test_overlapping_one_byte_inserts() {
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(100);

        // Insert a 2-byte value at 10
        h.insert(10, 2, "hello").unwrap();
        assert_eq!(h.entries(), 1);

        // We can insert before
        assert!(h.insert(8,  1, "ok").is_ok());
        assert_eq!(h.entries(), 2);
        assert!(h.insert(9,  1, "ok").is_ok());
        assert_eq!(h.entries(), 3);

        // We can't insert within
        assert!(h.insert(10, 1, "error").is_err());
        assert!(h.insert(11, 1, "error").is_err());
        assert_eq!(h.entries(), 3);

        // We can insert after
        assert!(h.insert(12, 1, "ok").is_ok());
        assert_eq!(h.entries(), 4);
        assert!(h.insert(13, 1, "ok").is_ok());
        assert_eq!(h.entries(), 5);
    }

    #[test]
    fn test_overlapping_multi_byte_inserts() {
        // Define 10-12, put something at 7-9 (good!)
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(100);
        h.insert(10, 3, "hello").unwrap();
        assert!(h.insert(7,  3, "ok").is_ok());

        // Define 10-12, try every overlapping bit
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(100);
        h.insert(10, 3, "hello").unwrap();
        assert!(h.insert(8,  3, "error").is_err());
        assert!(h.insert(9,  3, "error").is_err());
        assert!(h.insert(10, 3, "error").is_err());
        assert!(h.insert(11, 3, "error").is_err());
        assert!(h.insert(12, 3, "error").is_err());

        // 6-9 and 13-15 will work
        assert!(h.insert(6,  3, "ok").is_ok());
        assert!(h.insert(13, 3, "ok").is_ok());
        assert_eq!(h.entries(), 3);
    }

    #[test]
    fn test_remove() {
        // Define 10-12, put something at 7-9 (good!)
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(100);
        h.insert(8, 2, "hello").unwrap();
        h.insert(10, 2, "hello").unwrap();
        h.insert(12, 2, "hello").unwrap();
        assert_eq!(h.entries(), 3);

        h.remove(10);
        assert_eq!(h.entries(), 2);
        assert_eq!(h.get(10), None);
        assert_eq!(h.get(11), None);

        h.insert(10, 2, "hello").unwrap();
        assert_eq!(h.entries(), 3);

        h.remove(11);
        assert_eq!(h.entries(), 2);
        assert_eq!(h.get(10), None);
        assert_eq!(h.get(11), None);

        h.remove(13);
        assert_eq!(h.entries(), 1);
        assert_eq!(h.get(12), None);
        assert_eq!(h.get(13), None);

        h.remove(8);
        assert_eq!(h.entries(), 0);
        assert_eq!(h.get(8), None);
        assert_eq!(h.get(9), None);
    }

    #[test]
    fn beginning_works() {
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(10);
        h.insert(0, 2, "hello").unwrap();
        assert_eq!(h.entries(), 1);
        assert_eq!(h.get(0).unwrap(), (&"hello", 0, 2));
        assert_eq!(h.get(1).unwrap(), (&"hello", 0, 2));
        assert_eq!(h.get(2), None);
    }

    #[test]
    fn max_size() {
        // Inserting at 7-8-9 works
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(10);
        h.insert(7, 3, "hello").unwrap();
        assert_eq!(h.entries(), 1);

        // Inserting at 8-9-10 and onward does not
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(10);
        assert!(h.insert(8, 3, "hello").is_err());
        assert_eq!(h.entries(), 0);

        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(10);
        assert!(h.insert(9, 3, "hello").is_err());
        assert_eq!(h.entries(), 0);

        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(10);
        assert!(h.insert(10, 3, "hello").is_err());
        assert_eq!(h.entries(), 0);

        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(10);
        assert!(h.insert(11, 3, "hello").is_err());
        assert_eq!(h.entries(), 0);
    }

    #[test]
    fn test_iterator() {
        // Create a BumpyHashMap that looks like:
        //
        // [--0-- --1-- --2-- --3-- --4-- --5-- --6-- --7-- --8-- --9--]
        //        +-----------------            +----------------+
        //        |   "a" (2)| "b" |            |      "c"       |
        //        +----------+------            +----------------+
        let mut h: BumpyHashMap<&str> = BumpyHashMap::new(10);
        h.insert(1, 2, "a").unwrap();
        h.insert(3, 1, "b").unwrap();
        h.insert(6, 3, "c").unwrap();

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
