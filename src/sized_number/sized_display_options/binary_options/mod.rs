use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::sized_number::{GenericNumber, SizedOptions, SizedDisplay};

/// Configure display options for [`SizedDisplay::Binary`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BinaryOptions {
    /// Prefix binary strings with `0b`
    pub prefix: bool,

    /// Zero-pad binary strings to the full width - `00000001` vs `1`
    pub padded: bool,
}

impl BinaryOptions {
    pub fn new(prefix: bool, padded: bool) -> SizedDisplay {
        SizedDisplay::Binary(Self {
            prefix: prefix,
            padded: padded,
        })
    }

    pub fn pretty() -> SizedDisplay {
        Self::new(true, true)
    }
}

impl SizedOptions for BinaryOptions {
    fn to_string(&self, number: GenericNumber) -> SimpleResult<String> {
        let mut s = match (self.padded, number) {
            (true, GenericNumber::U8(v))   => format!("{:08b}", v),
            (true, GenericNumber::U16(v))  => format!("{:016b}", v),
            (true, GenericNumber::U32(v))  => format!("{:032b}", v),
            (true, GenericNumber::U64(v))  => format!("{:064b}", v),
            (true, GenericNumber::U128(v)) => format!("{:0128b}", v),
            (true, GenericNumber::I8(v))   => format!("{:08b}", v),
            (true, GenericNumber::I16(v))  => format!("{:016b}", v),
            (true, GenericNumber::I32(v))  => format!("{:032b}", v),
            (true, GenericNumber::I64(v))  => format!("{:064b}", v),
            (true, GenericNumber::I128(v)) => format!("{:0128b}", v),
            (true, GenericNumber::F32(_v)) => bail!("Cannot display floating point as binary"),
            (true, GenericNumber::F64(_v)) => bail!("Cannot display floating point as binary"),

            (false, GenericNumber::U8(v))   => format!("{:b}", v),
            (false, GenericNumber::U16(v))  => format!("{:b}", v),
            (false, GenericNumber::U32(v))  => format!("{:b}", v),
            (false, GenericNumber::U64(v))  => format!("{:b}", v),
            (false, GenericNumber::U128(v)) => format!("{:b}", v),
            (false, GenericNumber::I8(v))   => format!("{:b}", v),
            (false, GenericNumber::I16(v))  => format!("{:b}", v),
            (false, GenericNumber::I32(v))  => format!("{:b}", v),
            (false, GenericNumber::I64(v))  => format!("{:b}", v),
            (false, GenericNumber::I128(v)) => format!("{:b}", v),
            (false, GenericNumber::F32(_v)) => bail!("Cannot display floating point as binary"),
            (false, GenericNumber::F64(_v)) => bail!("Cannot display floating point as binary"),
        };

        // Add the prefix after for simplicity
        if self.prefix {
            s = format!("0b{}", s);
        }

        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use crate::sized_number::{Context, SizedDefinition};

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
            let number = SizedDefinition::U8.read(context)?;

            assert_eq!(
                expected,
                BinaryOptions::new(prefix, padded).to_string(number)?,
            );
        }

        Ok(())
    }
}
