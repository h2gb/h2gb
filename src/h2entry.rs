use std::fmt;
use h2value::H2Value;

#[derive(Debug)]
pub struct H2Entry {
  pub start:          usize,
  pub value:          H2Value,

  /* TODO: custom_display, comment, refs */
}

#[allow(dead_code)] // TODO: Remove
impl H2Entry {
  pub fn new(start: usize, value: H2Value) -> H2Entry {
    H2Entry {
      start,
      value,
    }
  }
}

impl fmt::Display for H2Entry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let start = self.start;
    write!(f, "{:08x} {}", start, self.value.to_string())
  }
}

impl PartialEq for H2Entry {
  fn eq(&self, other: &H2Entry) -> bool {
    self.start == other.start && self.value == other.value
  }
}
