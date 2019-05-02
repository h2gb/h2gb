use std::collections::HashMap;
use std::fmt;

use h2entry::H2Entry;
use h2value::H2Value;

#[derive(Debug)]
pub struct H2Buffer {
  memory: Box<[u8]>,
  entries: HashMap<usize, H2Entry>,
  name: String,
  virtual_start: usize,
}

#[allow(dead_code)] // TODO: Remove
impl H2Buffer {
  /// Return a simple, default single-byte entry for the given address
  fn default_entry(memory: &Box<[u8]>, i: usize) -> H2Entry {
    H2Entry::new(
      i,
      H2Value::u8(memory, i).unwrap(), // TODO: Fix the unwrap
    )
  }

  pub fn get(&self, address: usize) -> Option<&H2Entry> {
    if address >= self.memory.len() {
      return None;
    }

    let mut start = address;
    while !self.entries.contains_key(&start) && start > 0{
      start -= 1;
    }

    return self.entries.get(&start);
  }

  pub fn undefine(&mut self, address: usize, length: usize) {
    /* Get the start of the current entry */
    let start = match self.get(address) {
      Some(e) => e.start,
      None    => address,
    };

    /* Get the end of the last entry */
    let end = match self.get(address + length - 1) {
      Some(e) => e.start + e.value.length(),
      None    => address + length,
    };

    /* Loop from the left-most entry we could find to the end of the requested
       length */
    for i in start..end {
      /* Create a boring default entry */
      self.entries.insert(i, H2Buffer::default_entry(&self.memory, i));
    }
  }

  fn set(&mut self, offset: usize, value: H2Value) {
    let entry = H2Entry::new(
      offset,
      value,
    );

    let start = entry.start;

    /* This will undefine the starting address and anything "before" it */
    self.undefine(start, entry.value.length());

    /* Delete entries to make room for the object */
    for i in start..(start+entry.value.length()) {
      self.entries.remove(&i);
    }

    /* Insert the new entry */
    self.entries.insert(entry.start, entry);
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
    result.undefine(0, len);

    result
  }
}

impl fmt::Display for H2Buffer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut s = String::new();

    /* Get a sorted list of the addresses */
    let mut keys: Vec<&usize> = self.entries.keys().collect();
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
    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt8(0));

    assert_eq!(buffer.get(1).unwrap().start, 1);
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt8(1));

    assert_eq!(buffer.get(2).unwrap().start, 2);
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt8(2));

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt8(3));

    assert_eq!(buffer.get(4).unwrap().start, 4);
    assert_eq!(buffer.get(4).unwrap().value, H2Value::UInt8(100));

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
    let value = H2Value::u16(&buffer.memory, 1).unwrap();
    buffer.set(1, value);

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt8(0x00));

    assert_eq!(buffer.get(1).unwrap().start, 1);
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt16(0x0102));

    assert_eq!(buffer.get(2).unwrap().start, 1);
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt16(0x0102));

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt8(0x03));
  }

  #[test]
  fn test_set_overwrite_bigger_with_smaller() {
    let mut buffer = H2Buffer::new(Box::new([5, 6, 7, 8]));

    let value = H2Value::u32(&buffer.memory, 0).unwrap();
    buffer.set(0, value);

    let value = H2Value::u16(&buffer.memory, 1).unwrap();
    buffer.set(1, value);

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt8(0x05));

    assert_eq!(buffer.get(1).unwrap().start, 1);
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt16(0x0607));

    assert_eq!(buffer.get(2).unwrap().start, 1);
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt16(0x0607));

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt8(0x08));
  }

  #[test]
  fn test_set_with_overlap() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));


    let value = H2Value::u16(&buffer.memory, 0).unwrap();
    buffer.set(0, value); /* Overwritten */

    let value = H2Value::u32(&buffer.memory, 1).unwrap();
    buffer.set(1, value); /* Overwritten */

    let value = H2Value::u16(&buffer.memory, 2).unwrap();
    buffer.set(2, value); /* Overwritten */

    let value = H2Value::u16(&buffer.memory, 0).unwrap();
    buffer.set(0, value); /* Stays */

    let value = H2Value::u32(&buffer.memory, 4).unwrap();
    buffer.set(4, value); /* Stays */

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt16(0x0908));

    assert_eq!(buffer.get(1).unwrap().start, 0);
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt16(0x0908));

    assert_eq!(buffer.get(2).unwrap().start, 2);
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt16(0x0706));

    assert_eq!(buffer.get(3).unwrap().start, 2);
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt16(0x0706));

    assert_eq!(buffer.get(4).unwrap().start, 4);
    assert_eq!(buffer.get(4).unwrap().value, H2Value::UInt32(0x05040302));

    assert_eq!(buffer.get(5).unwrap().start, 4);
    assert_eq!(buffer.get(5).unwrap().value, H2Value::UInt32(0x05040302));

    assert_eq!(buffer.get(6).unwrap().start, 4);
    assert_eq!(buffer.get(6).unwrap().value, H2Value::UInt32(0x05040302));

    assert_eq!(buffer.get(7).unwrap().start, 4);
    assert_eq!(buffer.get(7).unwrap().value, H2Value::UInt32(0x05040302));

    assert_eq!(buffer.get(8).unwrap().start, 8);
    assert_eq!(buffer.get(8).unwrap().value, H2Value::UInt8(0x01));
  }

  #[test]
  fn test_bad_insert() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    /* This should simply do nothing */
    let value = H2Value::u8(&buffer.memory, 0).unwrap();
    buffer.set(100, value);
  }

  #[test]
  fn test_get_out_of_bounds() {
    let buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    /* This should return None */
    assert_eq!(buffer.get(100),  None);
  }

  #[test]
  fn test_undefine_inside_entry() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    let value = H2Value::u32(&buffer.memory, 1).unwrap();
    buffer.set(1, value);
    buffer.undefine(2, 1);

    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt8(0x09));
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt8(0x08));
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt8(0x07));
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt8(0x06));
    assert_eq!(buffer.get(4).unwrap().value, H2Value::UInt8(0x05));
    assert_eq!(buffer.get(5).unwrap().value, H2Value::UInt8(0x04));
  }

  #[test]
  fn test_undefine_touches_start() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    let value = H2Value::u32(&buffer.memory, 1).unwrap();
    buffer.set(1, value);
    buffer.undefine(0, 2);

    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt8(0x09));
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt8(0x08));
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt8(0x07));
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt8(0x06));
    assert_eq!(buffer.get(4).unwrap().value, H2Value::UInt8(0x05));
    assert_eq!(buffer.get(5).unwrap().value, H2Value::UInt8(0x04));
  }

  #[test]
  fn test_undefine_right_between_entries() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    let value = H2Value::u32(&buffer.memory, 0).unwrap();
    buffer.set(0, value);

    let value = H2Value::u32(&buffer.memory, 5).unwrap();
    buffer.set(5, value);

    buffer.undefine(4, 1);

    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt32(0x09080706));
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt32(0x09080706));
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt32(0x09080706));
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt32(0x09080706));
    assert_eq!(buffer.get(4).unwrap().value, H2Value::UInt8(0x05));
    assert_eq!(buffer.get(5).unwrap().value, H2Value::UInt32(0x04030201));
    assert_eq!(buffer.get(6).unwrap().value, H2Value::UInt32(0x04030201));
    assert_eq!(buffer.get(7).unwrap().value, H2Value::UInt32(0x04030201));
    assert_eq!(buffer.get(8).unwrap().value, H2Value::UInt32(0x04030201));
  }

  #[test]
  fn test_undefine_leaves_previous_and_next_entries() {
    let mut buffer = H2Buffer::new(Box::new([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));

    let value = H2Value::u16(&buffer.memory, 1).unwrap();
    buffer.set(1, value);

    let value = H2Value::u16(&buffer.memory, 3).unwrap();
    buffer.set(3, value);

    let value = H2Value::u16(&buffer.memory, 5).unwrap();
    buffer.set(5, value);

    buffer.undefine(3, 2);

    assert_eq!(buffer.get(0).unwrap().value, H2Value::UInt8(0x09));
    assert_eq!(buffer.get(1).unwrap().value, H2Value::UInt16(0x0807));
    assert_eq!(buffer.get(2).unwrap().value, H2Value::UInt16(0x0807));
    assert_eq!(buffer.get(3).unwrap().value, H2Value::UInt8(0x06));
    assert_eq!(buffer.get(4).unwrap().value, H2Value::UInt8(0x05));
    assert_eq!(buffer.get(5).unwrap().value, H2Value::UInt16(0x0403));
    assert_eq!(buffer.get(6).unwrap().value, H2Value::UInt16(0x0403));
  }
}
