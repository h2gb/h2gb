use std::fmt::Binary;
use std::mem;

use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::sized_number::SizedOptions;

/// Configure display options for [`SizedDisplay::Binary`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BinaryOptions {
    /// Prefix binary strings with `0b`
    pub prefix: bool,

    /// Zero-pad binary strings to the full width - `00000001` vs `1`
    pub padded: bool,
}

impl Default for BinaryOptions {
    fn default() -> Self {
        Self {
            padded: true,
            prefix: true,
        }
    }
}

impl BinaryOptions {
    /// An internal function to help with displaying binary
    pub fn to_s(self, v: Box<dyn Binary>) -> String {
        let v = v.as_ref();

        match (self.padded, self.prefix) {
            (false, false) => format!("{:b}", v),
            (false, true ) => format!("0b{:b}", v),
            (true, false) => {
                match mem::size_of_val(v) * 8 {
                    8   => format!("{:08b}",   v),
                    16  => format!("{:016b}",  v),
                    32  => format!("{:032b}",  v),
                    64  => format!("{:064b}",  v),
                    128 => format!("{:0128b}", v),
                    _   => format!("{:b}",     v),
                }
            },
            (true, true) => {
                match mem::size_of_val(v) * 8 {
                    8   => format!("0b{:08b}",   v),
                    16  => format!("0b{:016b}",  v),
                    32  => format!("0b{:032b}",  v),
                    64  => format!("0b{:064b}",  v),
                    128 => format!("0b{:0128b}", v),
                    _   => format!("0b{:b}",     v),
                }
            }
        }
    }
}

impl SizedOptions for BinaryOptions {
    fn to_s_i8(&self, v:   i8)   -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i16(&self, v:  i16)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i32(&self, v:  i32)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i64(&self, v:  i64)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i128(&self, v: i128) -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }


    fn to_s_u8(&self, v:   u8)   -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u16(&self, v:  u16)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u32(&self, v:  u32)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u64(&self, v:  u64)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u128(&self, v: u128) -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_f32(&self, _v:  f32) -> SimpleResult<String> {
        bail!("Floating point values cannot be displayed as binary")
    }

    fn to_s_f64(&self, _v:  f64) -> SimpleResult<String> {
        bail!("Floating point values cannot be displayed as binary")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use crate::sized_number::{Context, SizedDisplay, SizedDefinition};

    #[test]
    fn test_binary_i8() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\xab\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index    prefix      padded   expected
            (   0,      true,       true,    "0b00000000"),
            (   1,      true,       true,    "0b00000000"),
            (   2,      true,       true,    "0b00010010"),
            (   3,      true,       true,    "0b10101011"),
            (   4,      true,       true,    "0b11111111"),
            (   5,      true,       true,    "0b11111111"),

            (   0,      false,      false,   "0"),
            (   1,      false,      false,   "0"),
            (   2,      false,      false,   "10010"),
            (   3,      false,      false,   "10101011"),
            (   4,      false,      false,   "11111111"),
            (   5,      false,      false,   "11111111"),
        ];

        for (index, padded, prefix, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Binary(BinaryOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }
}
