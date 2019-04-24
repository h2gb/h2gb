use std::fmt;

#[derive(Debug)]
pub struct H2Entry {
  pub start: u64,
  pub length: u64,
  pub display: String,
  data_refs: Box<[u64]>,
  code_refs: Box<[u64]>,
}

#[allow(dead_code)] // TODO: Remove
impl H2Entry {
  // TODO: Handle out of range cleanly
  pub fn u8(memory: &Box<[u8]>, offset: u64) -> Option<H2Entry> {
    if offset >= memory.len() as u64 {
      return None;
    }

    let b = memory[offset as usize];

    Some(H2Entry {
      start: offset,
      length: 1,
      display: format!("db {:02x}", b),
      data_refs: Box::new([]),
      code_refs: Box::new([]),
    })
  }

  pub fn u16(memory: &Box<[u8]>, offset: u64) -> Option<H2Entry> {
    if offset + 1 >= memory.len() as u64 {
      return None;
    }

    let w: u16 = (memory[offset as usize] as u16) << 8 |
      (memory[(offset + 1) as usize] as u16);

    Some(H2Entry {
      start: offset,
      length: 2,
      display: format!("db {:04x}", w),
      data_refs: Box::new([]),
      code_refs: Box::new([]),
    })
  }

  pub fn u32(memory: &Box<[u8]>, offset: u64) -> Option<H2Entry> {
    if offset + 3 >= memory.len() as u64 {
      return None;
    }

    let d: u32 = (memory[offset as usize] as u32) << 24 |
           (memory[(offset + 1) as usize] as u32) << 16 |
           (memory[(offset + 2) as usize] as u32) << 8 |
           (memory[(offset + 3) as usize] as u32) << 0;

    Some(H2Entry {
      start: offset,
      length: 4,
      display: format!("db {:08x}", d),
      data_refs: Box::new([]),
      code_refs: Box::new([]),
    })
  }
}

impl fmt::Display for H2Entry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:08x} {}", self.start, self.display)
  }
}

impl PartialEq for H2Entry {
  fn eq(&self, other: &H2Entry) -> bool {
    self.start == other.start && self.length == other.length && self.display == other.display && self.data_refs == other.data_refs && self.code_refs == other.code_refs
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_u8() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let entry = H2Entry::u8(&memory, 0).unwrap();
    assert_eq!(entry.start,  0);
    assert_eq!(entry.length, 1);
    assert_eq!(entry.display, "db 00");

    let entry = H2Entry::u8(&memory, 4).unwrap();
    assert_eq!(entry.start,  4);
    assert_eq!(entry.length, 1);
    assert_eq!(entry.display, "db 64");
  }

  #[test]
  fn test_out_of_bounds_u8() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let entry = H2Entry::u8(&memory, 5);
    assert_eq!(entry.is_none(), true);
  }

  #[test]
  fn test_u16() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let entry = H2Entry::u16(&memory, 0).unwrap();
    assert_eq!(entry.start,  0);
    assert_eq!(entry.length, 2);
    assert_eq!(entry.display, "db 0001");

    let entry = H2Entry::u16(&memory, 3).unwrap();
    assert_eq!(entry.start,  3);
    assert_eq!(entry.length, 2);
    assert_eq!(entry.display, "db 0364");
  }

  #[test]
  fn test_out_of_bounds_u16() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let entry = H2Entry::u16(&memory, 4);
    assert_eq!(entry.is_none(), true);

    let entry = H2Entry::u16(&memory, 100);
    assert_eq!(entry.is_none(), true);
  }

  #[test]
  fn test_u32() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let entry = H2Entry::u32(&memory, 0).unwrap();
    assert_eq!(entry.start,  0);
    assert_eq!(entry.length, 4);
    assert_eq!(entry.display, "db 00010203");

    let entry = H2Entry::u32(&memory, 1).unwrap();
    assert_eq!(entry.start,  1);
    assert_eq!(entry.length, 4);
    assert_eq!(entry.display, "db 01020364");
  }

  #[test]
  fn test_out_of_bounds_u32() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let entry = H2Entry::u32(&memory, 2);
    assert_eq!(entry.is_none(), true);

    let entry = H2Entry::u32(&memory, 3);
    assert_eq!(entry.is_none(), true);

    let entry = H2Entry::u32(&memory, 4);
    assert_eq!(entry.is_none(), true);

    let entry = H2Entry::u32(&memory, 100);
    assert_eq!(entry.is_none(), true);
  }
}
