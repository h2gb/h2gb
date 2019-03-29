use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
struct H2Entry {
  start: u64,
  length: u64,
  display: String,
  data_refs: Box<[u64]>,
  code_refs: Box<[u64]>,
}

impl H2Entry {
  fn default(start: u64, b: u8) -> H2Entry {
    return H2Entry {
      start: start,
      length: 1,
      display: format!("db {:02}", b),
      data_refs: Box::new([]),
      code_refs: Box::new([]),
    };
  }
}

impl fmt::Display for H2Entry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.start, self.display)
  }
}

impl PartialEq for H2Entry {
  fn eq(&self, other: &H2Entry) -> bool {
    self.start == other.start && self.length == other.length && self.display == other.display && self.data_refs == other.data_refs && self.code_refs == other.code_refs
  }
}


#[derive(Debug)]
struct H2Buffer {
  memory: Box<[u8]>,
  entries: HashMap<u64, H2Entry>,
  name: String,
  virtual_start: u64,
}

impl H2Buffer {
  pub fn new(memory: Box<[u8]>) -> H2Buffer {
    let mut entries: HashMap<u64, H2Entry> = HashMap::new();
    for (i, b) in memory.iter().enumerate() {
      entries.insert(i as u64, H2Entry::default(i as u64, *b));
    }

    H2Buffer {
      memory: memory,
      entries: entries,
      name: String::from("my name"),
      virtual_start: 0,
    }
  }

  pub fn set(&mut self, entry: H2Entry) {
    let start = entry.start;
    self.delete(start, entry.length);
    self.entries.insert(entry.start, entry);
  }

  pub fn delete(&mut self, address: u64, length: u64) {
    /* If there's an object to the 'left', find it. */
    let mut start = address;
    while !self.entries.contains_key(&start) {
      start -= 1;
    }
    self.entries.insert(start as u64, H2Entry::default(start, self.memory[start as usize]));

    /* Remove the rest of the entries */
    for i in address..(address+length) {
      self.entries.insert(i as u64, H2Entry::default(i, self.memory[i as usize]));
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

  pub fn print(&self) {
    for (_address, entry) in &self.entries {
      println!("{}", entry);
    }
  }
}

fn main() {
  let mut buffer = H2Buffer::new(Box::new([0, 1, 2, 3, 4, 5]));

  let entry = H2Entry {
    start: 2,
    length: 3,
    display: String::from("test"),
    data_refs: Box::new([]),
    code_refs: Box::new([]),
  };
  buffer.set(entry);

  buffer.print();
  println!();
  println!("Zero:");
  println!("{}", buffer.get(0).unwrap());
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_create() {
    let buffer = H2Buffer::new(Box::new([0, 1, 2, 3]));

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

    assert_eq!(buffer.get(4), None);
  }

  #[test]
  fn test_set() {
    let mut buffer = H2Buffer::new(Box::new([0, 1, 2, 3]));

    buffer.set(H2Entry {
      start: 1,
      length: 2,
      display: String::from("test"),
      data_refs: Box::new([]),
      code_refs: Box::new([]),
    });

    assert_eq!(buffer.get(0).unwrap().start, 0);
    assert_eq!(buffer.get(0).unwrap().length, 1);
    assert_eq!(buffer.get(0).unwrap().display, "db 00");

    for i in 1..3 {
      assert_eq!(buffer.get(i).unwrap().start, 1);
      assert_eq!(buffer.get(i).unwrap().length, 2);
      assert_eq!(buffer.get(i).unwrap().display, "test");
    }

    assert_eq!(buffer.get(3).unwrap().start, 3);
    assert_eq!(buffer.get(3).unwrap().length, 1);
    assert_eq!(buffer.get(3).unwrap().display, "db 03");

    assert_eq!(buffer.get(4), None);
  }

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

    buffer.print();

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
  }
}
