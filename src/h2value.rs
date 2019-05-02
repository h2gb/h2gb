use std::fmt;

#[allow(dead_code)] // TODO: Remove
#[derive(Debug)]
#[derive(PartialEq)]
pub enum H2Value { /* TODO: add endian */
  UInt8(u8),
  UInt16(u16),
  UInt32(u32),
  UInt64(u64),
  NTString(String),
}

#[allow(dead_code)] // TODO: Remove
impl H2Value {
  // TODO: Handle out of range cleanly
  pub fn u8(memory: &Box<[u8]>, offset: usize) -> Option<H2Value> {
    if offset >= memory.len() {
      return None;
    }

    let b = memory[offset];

    Some(H2Value::UInt8(b))
  }

  pub fn u16(memory: &Box<[u8]>, offset: usize) -> Option<H2Value> {
    if offset + 1 >= memory.len() {
      return None;
    }

    let w: u16 = (memory[offset] as u16) << 8 |
      (memory[(offset + 1)] as u16);

    Some(H2Value::UInt16(w))
  }

  pub fn u32(memory: &Box<[u8]>, offset: usize) -> Option<H2Value> {
    if offset + 3 >= memory.len() {
      return None;
    }

    let d: u32 = (memory[offset] as u32) << 24 |
           (memory[(offset + 1)] as u32) << 16 |
           (memory[(offset + 2)] as u32) << 8 |
           (memory[(offset + 3)] as u32) << 0;

    Some(H2Value::UInt32(d))
  }

  pub fn length(&self) -> usize {
    match self {
      H2Value::UInt8(_)        => 1,
      H2Value::UInt16(_)       => 2,
      H2Value::UInt32(_)       => 4,
      H2Value::UInt64(_)       => 8,
      H2Value::NTString(value) => value.len(),
    }
  }
}

impl fmt::Display for H2Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let test = match self {
      H2Value::UInt8(value)    => format!("db {:02x}", value),
      H2Value::UInt16(value)   => format!("dw {:04x}", value),
      H2Value::UInt32(value)   => format!("dd {:08x}", value),
      H2Value::UInt64(value)   => format!("dq {:016x}", value),
      H2Value::NTString(value) => format!("\"{}\"", value),
    };

    write!(f, "{}", test)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_u8() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let value = H2Value::u8(&memory, 0).unwrap();
    assert_eq!(value, H2Value::UInt8(0));
    assert_eq!(value.to_string(), String::from("db 00"));

    let value = H2Value::u8(&memory, 4).unwrap();
    assert_eq!(value, H2Value::UInt8(100));
    assert_eq!(value.to_string(), String::from("db 64"));
  }

  #[test]
  fn test_out_of_bounds_u8() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let value = H2Value::u8(&memory, 5);
    assert!(value.is_none());
  }

  #[test]
  fn test_u16() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let value = H2Value::u16(&memory, 0).unwrap();
    assert_eq!(value, H2Value::UInt16(0x0001));
    assert_eq!(value.to_string(), String::from("dw 0001"));

    let value = H2Value::u16(&memory, 3).unwrap();
    assert_eq!(value, H2Value::UInt16(0x0364));
    assert_eq!(value.to_string(), String::from("dw 0364"));
  }

  #[test]
  fn test_out_of_bounds_u16() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let value = H2Value::u16(&memory, 4);
    assert!(value.is_none());

    let value = H2Value::u16(&memory, 100);
    assert!(value.is_none());
  }

  #[test]
  fn test_u32() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let value = H2Value::u32(&memory, 0).unwrap();
    assert_eq!(value, H2Value::UInt32(0x00010203));
    assert_eq!(value.to_string(), String::from("dd 00010203"));

    let value = H2Value::u32(&memory, 1).unwrap();
    assert_eq!(value, H2Value::UInt32(0x01020364));
    assert_eq!(value.to_string(), String::from("dd 01020364"));
  }

  #[test]
  fn test_out_of_bounds_u32() {
    let memory: Box<[u8]> = Box::new([0, 1, 2, 3, 0x64]);

    let value = H2Value::u32(&memory, 2);
    assert!(value.is_none());

    let value = H2Value::u32(&memory, 3);
    assert!(value.is_none());

    let value = H2Value::u32(&memory, 4);
    assert!(value.is_none());

    let value = H2Value::u32(&memory, 100);
    assert!(value.is_none());
  }
}
