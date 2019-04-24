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

  pub fn new(memory: Box<[u8]>) -> H2Buffer {
    /* Create a hashmap of entries - this is kind of the core of the
       abstraction */
    let mut entries: HashMap<u64, H2Entry> = HashMap::new();

    /* Create default entries */
    /* TODO: Can I just use a big delete()? */
    for (i, _b) in memory.iter().enumerate() {
      entries.insert(i as u64, H2Buffer::default_entry(&memory, i as u64));
    }

    /* Create the buffer */
    H2Buffer {
      memory: memory,
      entries: entries,
      name: String::from("my name"),
      virtual_start: 0,
    }
  }

  fn set(&mut self, entry: H2Entry) {
    let start = entry.start;
    self.delete(start, entry.length);
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

  pub fn delete(&mut self, address: u64, length: u64) {
    /* If there's an object to the 'left', find it. */
    /* TODO: I think this will go too far left though? */
    let mut start = address;
    while !self.entries.contains_key(&start) {
      start -= 1;
    }
    self.entries.insert(start, H2Buffer::default_entry(&self.memory, start));

    /* Remove the rest of the entries */
    for i in address..(address+length) {
      self.entries.insert(i as u64, H2Buffer::default_entry(&self.memory, start));
    }
  }

  pub fn get(&self, address: u64) -> Option<&H2Entry> {
    if address >= self.memory.len() as u64 {
      return None;
    }

    let mut start = address;
    while !self.entries.contains_key(&start) {
      start -= 1;
    }

    return Some(&self.entries.get(&start).unwrap());
  }
}

impl fmt::Display for H2Buffer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut s = String::new();

    for (_address, entry) in &self.entries {
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
  fn test_set() {
    let mut buffer = H2Buffer::new(Box::new([0, 1, 2, 3]));

    println!("Before:");
    println!("{}", buffer);
    buffer.set_u16(1);
    println!();
    println!("After:");
    println!("{}", buffer);
    println!();

    println!("0: {}", buffer.get(0).unwrap());
    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().length, 1);
    assert_eq!(buffer.get(0).unwrap().display, "db 00");

    println!("1: {}", buffer.get(1).unwrap());
    assert_eq!(buffer.get(1).unwrap().start, 1);
    assert_eq!(buffer.get(1).unwrap().length, 2);
    assert_eq!(buffer.get(1).unwrap().display, "db 0102");

    println!("2: {}", buffer.get(2).unwrap());
    assert_eq!(buffer.get(2).unwrap().start, 1);
    assert_eq!(buffer.get(2).unwrap().length, 2);
    assert_eq!(buffer.get(2).unwrap().display, "db 0102");

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().length, 1);
    assert_eq!(buffer.get(3).unwrap().display, "db 03");
  }

  /*
  #[test]
  fn test_set_with_overlap() {
    let mut buffer = H2Buffer::new(Box::new([0, 1, 2, 3]));

    buffer.set(H2Entry {
      start: 1,
      length: 2,
      display: String::from("test"),
      data_refs: Box::new([]),
      code_refs: Box::new([]),
    });

    buffer.set(H2Entry {
      start: 2,
      length: 2,
      display: String::from("test"),
      data_refs: Box::new([]),
      code_refs: Box::new([]),
    });

    println!("{}", buffer);

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().length, 1);
    assert_eq!(buffer.get(0).unwrap().display, "db 00");

    for i in 2..4 {
      assert_eq!(buffer.get(i).unwrap().start, i);
      assert_eq!(buffer.get(i).unwrap().length, 1);
      assert_eq!(buffer.get(i).unwrap().display, "test");
    }

    assert_eq!(buffer.get(1).unwrap().start, 3);
    assert_eq!(buffer.get(1).unwrap().length, 1);
    assert_eq!(buffer.get(1).unwrap().display, "db 03");
  }

  #[test]
  fn test_bad_insert() {
  }

  #[test]
  fn test_get_out_of_bounds() {
  }

  #[test]
  fn test_delete() {
  }*/
}
