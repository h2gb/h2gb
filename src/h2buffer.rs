use std::collections::HashMap;
use std::fmt;

use h2entry::H2Entry;

#[derive(Debug)]
pub struct H2Buffer {
  memory: Box<[u8]>,
  entries: HashMap<u64, H2Entry>,
  name: String,
  virtual_start: u64,
}

#[allow(dead_code)] // TODO: Remove
impl H2Buffer {
  /// Return a simple, default single-byte entry for the given address
  fn default_entry(memory: &Box<[u8]>, i: u64) -> H2Entry {
    H2Entry::u8(memory, i).unwrap() // TODO
  }

  pub fn get(&self, address: u64) -> Option<&H2Entry> {
    if address >= self.memory.len() as u64 {
      return None;
    }

    let mut start = address;
    while !self.entries.contains_key(&start) && start > 0{
      start -= 1;
    }

    return self.entries.get(&start);
  }

  pub fn undefine(&mut self, address: u64, length: u64) {
    /* Get the start of the current entry */
    let start = match self.get(address) {
      Some(e) => e.start,
      None    => address,
    };

    /* Get the end of the last entry */
    let end = match self.get(address + length - 1) {
      Some(e) => e.start + e.length,
      None    => address + length,
    };

    /* Loop from the left-most entry we could find to the end of the requested
       length */
    for i in start..end {
      /* Create a boring default entry */
      self.entries.insert(i, H2Buffer::default_entry(&self.memory, i));
    }
  }

  fn set(&mut self, entry: H2Entry) {
    let start = entry.start;

    /* This will undefine the starting address and anything "before" it */
    self.undefine(start, entry.length);

    /* Delete entries to make room for the object */
    for i in start..(start+entry.length) {
      self.entries.remove(&i);
    }

    /* Insert the new entry */
    self.entries.insert(entry.start, entry);
  }

  pub fn set_u8(&mut self, offset: u64) {
    H2Entry::u8(&self.memory, offset).map(|entry| {
      self.set(entry);
    });
  }

  pub fn set_u16(&mut self, offset: u64) {
    H2Entry::u16(&self.memory, offset).map(|entry| {
      self.set(entry);
    });
  }

  pub fn set_u32(&mut self, offset: u64) {
    H2Entry::u32(&self.memory, offset).map(|entry| {
      self.set(entry);
    });
  }

  pub fn new(memory: Box<[u8]>) -> H2Buffer {
    /* Create the buffer */
    let mut result = H2Buffer {
      memory: memory,
      entries: HashMap::new(),
      name: String::from("my name"),
      virtual_start: 0,
    };

    /* Populate it with "nothing" */
    let len = result.memory.len();
    result.undefine(0, len as u64);

    result
  }
}

impl fmt::Display for H2Buffer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut s = String::new();

    /* Get a sorted list of the addresses */
    let mut keys: Vec<&u64> = self.entries.keys().collect();
    keys.sort();
    let keys = keys;

    for address in keys {
      let entry = &self.entries[address];
      s.push_str(&format!("{}\n", entry));
    }

    write!(f, "{}", s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_create() {
    let buffer = H2Buffer::new(Box::new([0, 1, 2, 3, 0x64]));

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().length, 1);
    assert_eq!(buffer.get(0).unwrap().display, "db 00");

    assert_eq!(buffer.get(1).unwrap().start, 1);
    assert_eq!(buffer.get(1).unwrap().length, 1);
    assert_eq!(buffer.get(1).unwrap().display, "db 01");

    assert_eq!(buffer.get(2).unwrap().start, 2);
    assert_eq!(buffer.get(2).unwrap().length, 1);
    assert_eq!(buffer.get(2).unwrap().display, "db 02");

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().length, 1);
    assert_eq!(buffer.get(3).unwrap().display, "db 03");

    assert_eq!(buffer.get(4).unwrap().start, 4);
    assert_eq!(buffer.get(4).unwrap().length, 1);
    assert_eq!(buffer.get(4).unwrap().display, "db 64");

    assert_eq!(buffer.get(5),  None);
    assert_eq!(buffer.get(6),  None);
    assert_eq!(buffer.get(7),  None);
    assert_eq!(buffer.get(8),  None);
    assert_eq!(buffer.get(9),  None);
    assert_eq!(buffer.get(10), None);
  }

  #[test]
  fn test_basic_set() {
    let mut buffer = H2Buffer::new(Box::new([0, 1, 2, 3]));

    buffer.set_u16(1);

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().length, 1);
    assert_eq!(buffer.get(0).unwrap().display, "db 00");

    assert_eq!(buffer.get(1).unwrap().start, 1);
    assert_eq!(buffer.get(1).unwrap().length, 2);
    assert_eq!(buffer.get(1).unwrap().display, "db 0102");

    assert_eq!(buffer.get(2).unwrap().start, 1);
    assert_eq!(buffer.get(2).unwrap().length, 2);
    assert_eq!(buffer.get(2).unwrap().display, "db 0102");

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().length, 1);
    assert_eq!(buffer.get(3).unwrap().display, "db 03");
  }

  #[test]
  fn test_set_overwrite_bigger_with_smaller() {
    let mut buffer = H2Buffer::new(Box::new([5, 6, 7, 8]));

    buffer.set_u32(0);
    buffer.set_u16(1);

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().length, 1);
    assert_eq!(buffer.get(0).unwrap().display, "db 05");

    assert_eq!(buffer.get(1).unwrap().start, 1);
    assert_eq!(buffer.get(1).unwrap().length, 2);
    assert_eq!(buffer.get(1).unwrap().display, "db 0607");

    assert_eq!(buffer.get(2).unwrap().start, 1);
    assert_eq!(buffer.get(2).unwrap().length, 2);
    assert_eq!(buffer.get(2).unwrap().display, "db 0607");

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().length, 1);
    assert_eq!(buffer.get(3).unwrap().display, "db 08");
  }

  #[test]
  fn test_set_with_overlap() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    buffer.set_u16(0); /* Overwritten */
    buffer.set_u32(1); /* Overwritten */
    buffer.set_u16(2); /* Overwritten */
    buffer.set_u16(0); /* Stays */
    buffer.set_u32(4); /* Stays */

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().length, 2);
    assert_eq!(buffer.get(0).unwrap().display, "db 0908");

    assert_eq!(buffer.get(1).unwrap().start, 0);
    assert_eq!(buffer.get(1).unwrap().length, 2);
    assert_eq!(buffer.get(1).unwrap().display, "db 0908");

    assert_eq!(buffer.get(2).unwrap().start, 2);
    assert_eq!(buffer.get(2).unwrap().length, 2);
    assert_eq!(buffer.get(2).unwrap().display, "db 0706");

    assert_eq!(buffer.get(3).unwrap().start, 2);
    assert_eq!(buffer.get(3).unwrap().length, 2);
    assert_eq!(buffer.get(3).unwrap().display, "db 0706");

    assert_eq!(buffer.get(4).unwrap().start, 4);
    assert_eq!(buffer.get(4).unwrap().length, 4);
    assert_eq!(buffer.get(4).unwrap().display, "db 05040302");

    assert_eq!(buffer.get(5).unwrap().start, 4);
    assert_eq!(buffer.get(5).unwrap().length, 4);
    assert_eq!(buffer.get(5).unwrap().display, "db 05040302");

    assert_eq!(buffer.get(6).unwrap().start, 4);
    assert_eq!(buffer.get(6).unwrap().length, 4);
    assert_eq!(buffer.get(6).unwrap().display, "db 05040302");

    assert_eq!(buffer.get(7).unwrap().start, 4);
    assert_eq!(buffer.get(7).unwrap().length, 4);
    assert_eq!(buffer.get(7).unwrap().display, "db 05040302");

    assert_eq!(buffer.get(8).unwrap().start, 8);
    assert_eq!(buffer.get(8).unwrap().length, 1);
    assert_eq!(buffer.get(8).unwrap().display, "db 01");
  }

  #[test]
  fn test_bad_insert() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    /* This should simply do nothing */
    buffer.set_u16(100);
  }

  #[test]
  fn test_get_out_of_bounds() {
    let buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    /* This should return None */
    assert_eq!(buffer.get(100),  None);
  }

  #[test]
  fn test_undefine_inside_object() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    buffer.set_u32(1);
    buffer.undefine(2, 1);

    assert_eq!(buffer.get(0).unwrap().display, "db 09");
    assert_eq!(buffer.get(1).unwrap().display, "db 08");
    assert_eq!(buffer.get(2).unwrap().display, "db 07");
    assert_eq!(buffer.get(3).unwrap().display, "db 06");
    assert_eq!(buffer.get(4).unwrap().display, "db 05");
    assert_eq!(buffer.get(5).unwrap().display, "db 04");
  }

  #[test]
  fn test_undefine_touches_start() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    buffer.set_u32(1);
    buffer.undefine(0, 2);

    assert_eq!(buffer.get(0).unwrap().display, "db 09");
    assert_eq!(buffer.get(1).unwrap().display, "db 08");
    assert_eq!(buffer.get(2).unwrap().display, "db 07");
    assert_eq!(buffer.get(3).unwrap().display, "db 06");
    assert_eq!(buffer.get(4).unwrap().display, "db 05");
    assert_eq!(buffer.get(5).unwrap().display, "db 04");
  }

  #[test]
  fn test_undefine_right_between_entries() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    buffer.set_u32(0);
    buffer.set_u32(5);
    buffer.undefine(4, 1);

    assert_eq!(buffer.get(0).unwrap().display, "db 09080706");
    assert_eq!(buffer.get(1).unwrap().display, "db 09080706");
    assert_eq!(buffer.get(2).unwrap().display, "db 09080706");
    assert_eq!(buffer.get(3).unwrap().display, "db 09080706");
    assert_eq!(buffer.get(4).unwrap().display, "db 05");
    assert_eq!(buffer.get(5).unwrap().display, "db 04030201");
    assert_eq!(buffer.get(6).unwrap().display, "db 04030201");
    assert_eq!(buffer.get(7).unwrap().display, "db 04030201");
    assert_eq!(buffer.get(8).unwrap().display, "db 04030201");
  }

  #[test]
  fn test_undefine_leaves_previous_and_next_entries() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    buffer.set_u16(1);
    buffer.set_u16(3);
    buffer.set_u16(5);
    buffer.undefine(3, 2);

    assert_eq!(buffer.get(0).unwrap().display, "db 09");
    assert_eq!(buffer.get(1).unwrap().display, "db 0807");
    assert_eq!(buffer.get(2).unwrap().display, "db 0807");
    assert_eq!(buffer.get(3).unwrap().display, "db 06");
    assert_eq!(buffer.get(4).unwrap().display, "db 05");
    assert_eq!(buffer.get(5).unwrap().display, "db 0403");
    assert_eq!(buffer.get(6).unwrap().display, "db 0403");
  }
}
