use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::datatype::helpers::H2Context;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Format {
    Hex,
    HexUppercase,
    DecimalSigned,
    DecimalUnsigned,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Size {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberDefinition {
    format: Format,
    size: Size,
    endian: Endian,
}

impl From<(Format, Size, Endian)> for NumberDefinition {
    fn from(o: (Format, Size, Endian)) -> NumberDefinition {
        Self {
            format: o.0,
            size: o.1,
            endian: o.2,
        }
    }
}

impl NumberDefinition {
    pub fn new(format: Format, size: Size, endian: Endian) -> Self {
        Self {
            format: format,
            size:   size,
            endian: endian,
        }
    }

    pub fn u8() -> Self {
        Self::new(Format::Hex, Size::Eight, Endian::Big)
    }

    pub fn u16_big() -> Self {
        Self::new(Format::Hex, Size::Sixteen, Endian::Big)
    }

    pub fn u16_little() -> Self {
        Self::new(Format::Hex, Size::Sixteen, Endian::Little)
    }

    pub fn u32_big() -> Self {
        Self::new(Format::Hex, Size::ThirtyTwo, Endian::Big)
    }

    pub fn u32_little() -> Self {
        Self::new(Format::Hex, Size::ThirtyTwo, Endian::Little)
    }

    pub fn u64_big() -> Self {
        Self::new(Format::Hex, Size::SixtyFour, Endian::Big)
    }

    pub fn u64_little() -> Self {
        Self::new(Format::Hex, Size::SixtyFour, Endian::Little)
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        // Get the length and make sure it's actually possible
        if context.index + self.len() > context.data.len() {
            bail!("Overflow");
        }

        // A 64-bit value is enough to cover all contingencies
        let mut n: u64 = 0;

        // Loop through as much as we need to
        for i in 0..self.len() {
            match self.endian {
                // Loop forward through the bytes for big endian
                Endian::Big => {
                    n = (n << 8) | context.data[context.index + i] as u64;
                },

                // Loop backward through the bytes for little endian
                Endian::Little => {
                    n = (n << 8) | context.data[context.index + self.len() - i - 1] as u64;
                },
            };
        }

        match self.size {
            Size::Eight => {
                Ok(match self.format {
                    Format::Hex             => format!("{:02x}", n as u8),
                    Format::HexUppercase    => format!("{:02X}", n as u8),
                    Format::DecimalUnsigned => format!("{}",     n as u8),
                    Format::DecimalSigned   => format!("{}",     n as i8),
                }.to_string())
            },
            Size::Sixteen => {
                Ok(match self.format {
                    Format::Hex             => format!("{:04x}", n as u16),
                    Format::HexUppercase    => format!("{:04X}", n as u16),
                    Format::DecimalUnsigned => format!("{}",     n as u16),
                    Format::DecimalSigned   => format!("{}",     n as i16),
                }.to_string())
            },
            Size::ThirtyTwo => {
                Ok(match self.format {
                    Format::Hex             => format!("{:08x}", n as u32),
                    Format::HexUppercase    => format!("{:08X}", n as u32),
                    Format::DecimalUnsigned => format!("{}",     n as u32),
                    Format::DecimalSigned   => format!("{}",     n as i32),
                }.to_string())
            },
            Size::SixtyFour => {
                Ok(match self.format {
                    Format::Hex             => format!("{:016x}", n as u64),
                    Format::HexUppercase    => format!("{:016X}", n as u64),
                    Format::DecimalUnsigned => format!("{}",      n as u64),
                    Format::DecimalSigned   => format!("{}",      n as i64),
                }.to_string())
            },
        }
    }

    pub fn len(&self) -> usize {
        match self.size {
            Size::Eight => 1,
            Size::Sixteen => 2,
            Size::ThirtyTwo => 4,
            Size::SixtyFour => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    #[test]
    fn test_number_to_string() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFFABCD\x80AAABBBB".to_vec();

        let tests = vec![
            // Index  Expected  Definition

            // 8-bit hex
            (  0,     "00",     (Format::Hex, Size::Eight, Endian::Big) ),
            (  1,     "7f",     (Format::Hex, Size::Eight, Endian::Big) ),
            (  2,     "80",     (Format::Hex, Size::Eight, Endian::Big) ),
            (  3,     "ff",     (Format::Hex, Size::Eight, Endian::Big) ),
            (  3,     "FF",     (Format::HexUppercase, Size::Eight, Endian::Big) ),

            // 8-bit unsigned
            (  0,     "0",      (Format::DecimalUnsigned, Size::Eight, Endian::Big) ),
            (  1,     "127",    (Format::DecimalUnsigned, Size::Eight, Endian::Big) ),
            (  2,     "128",    (Format::DecimalUnsigned, Size::Eight, Endian::Big) ),
            (  3,     "255",    (Format::DecimalUnsigned, Size::Eight, Endian::Big) ),

            // 8-bit signed
            (  0,     "0",      (Format::DecimalSigned, Size::Eight, Endian::Big) ),
            (  1,     "127",    (Format::DecimalSigned, Size::Eight, Endian::Big) ),
            (  2,     "-128",   (Format::DecimalSigned, Size::Eight, Endian::Big) ),
            (  3,     "-1",     (Format::DecimalSigned, Size::Eight, Endian::Big) ),

            // 16-bit, big endian, hex
            (  0,     "007f",   (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  1,     "7f80",   (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  2,     "80ff",   (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  3,     "ff41",   (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  3,     "FF41",   (Format::HexUppercase, Size::Sixteen, Endian::Big) ),

            // 16-bit, little endian, hex
            (  0,     "7f00",   (Format::Hex, Size::Sixteen, Endian::Little) ),
            (  1,     "807f",   (Format::Hex, Size::Sixteen, Endian::Little) ),
            (  2,     "ff80",   (Format::Hex, Size::Sixteen, Endian::Little) ),
            (  3,     "41ff",   (Format::Hex, Size::Sixteen, Endian::Little) ),

            // 16-bit, big endian, unsigned
            (  0,     "127",    (Format::DecimalUnsigned, Size::Sixteen, Endian::Big) ),
            (  1,     "32640",  (Format::DecimalUnsigned, Size::Sixteen, Endian::Big) ),
            (  2,     "33023",  (Format::DecimalUnsigned, Size::Sixteen, Endian::Big) ),
            (  3,     "65345",  (Format::DecimalUnsigned, Size::Sixteen, Endian::Big) ),

            // 16-bit, little endian, unsigned
            (  0,     "32512",  (Format::DecimalUnsigned, Size::Sixteen, Endian::Little) ),
            (  1,     "32895",  (Format::DecimalUnsigned, Size::Sixteen, Endian::Little) ),
            (  2,     "65408",  (Format::DecimalUnsigned, Size::Sixteen, Endian::Little) ),
            (  3,     "16895",  (Format::DecimalUnsigned, Size::Sixteen, Endian::Little) ),

            // 16-bit, big endian, signed
            (  0,     "127",    (Format::DecimalSigned, Size::Sixteen, Endian::Big) ),
            (  1,     "32640",  (Format::DecimalSigned, Size::Sixteen, Endian::Big) ),
            (  2,     "-32513", (Format::DecimalSigned, Size::Sixteen, Endian::Big) ),
            (  3,     "-191",   (Format::DecimalSigned, Size::Sixteen, Endian::Big) ),

            // 16-bit, little endian, signed
            (  0,     "32512",   (Format::DecimalSigned, Size::Sixteen, Endian::Little) ),
            (  1,     "-32641",  (Format::DecimalSigned, Size::Sixteen, Endian::Little) ),
            (  2,     "-128",    (Format::DecimalSigned, Size::Sixteen, Endian::Little) ),
            (  3,     "16895",   (Format::DecimalSigned, Size::Sixteen, Endian::Little) ),

            // 32-bit, big / little endian, hex
            (  0,     "007f80ff", (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  0,     "ff807f00", (Format::Hex, Size::ThirtyTwo, Endian::Little) ),

            // 32-bit, big / little endian, unsigned
            (  0,     "8356095",    (Format::DecimalUnsigned, Size::ThirtyTwo, Endian::Big) ),
            (  0,     "4286611200", (Format::DecimalUnsigned, Size::ThirtyTwo, Endian::Little) ),

            // 32-bit, big / little endian, signed
            (  0,     "8356095",  (Format::DecimalSigned, Size::ThirtyTwo, Endian::Big) ),
            (  0,     "-8356096", (Format::DecimalSigned, Size::ThirtyTwo, Endian::Little) ),

            // 64-bit, big / little endian, hex
            (  0,     "007f80ff41424344", (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  8,     "8041414142424242", (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  0,     "44434241ff807f00", (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  8,     "4242424241414180", (Format::Hex, Size::SixtyFour, Endian::Little) ),

            // 64-bit, big / little endian, unsigned
            (  0,     "35889155842130756",   (Format::DecimalUnsigned, Size::SixtyFour, Endian::Big) ),
            (  8,     "9241739658881286722", (Format::DecimalUnsigned, Size::SixtyFour, Endian::Big) ),
            (  0,     "4918848069245632256", (Format::DecimalUnsigned, Size::SixtyFour, Endian::Little) ),
            (  8,     "4774451407296217472", (Format::DecimalUnsigned, Size::SixtyFour, Endian::Little) ),

            // // 64-bit, big / little endian, signed
            // (  0,     "8356095",  (Format::DecimalSigned, Size::SixtyFour, Endian::Big) ),
            // (  0,     "-8356096", (Format::DecimalSigned, Size::SixtyFour, Endian::Little) ),
            (  0,     "35889155842130756",    (Format::DecimalSigned, Size::SixtyFour, Endian::Big) ),
            (  8,     "-9205004414828264894", (Format::DecimalSigned, Size::SixtyFour, Endian::Big) ),
            (  0,     "4918848069245632256",  (Format::DecimalSigned, Size::SixtyFour, Endian::Little) ),
            (  8,     "4774451407296217472",  (Format::DecimalSigned, Size::SixtyFour, Endian::Little) ),
        ];

        for (index, expected, definition) in tests {
            let d = NumberDefinition::from(definition);
            assert_eq!(expected.to_string(), d.to_string(&(&data, index).into())?);
        }

        Ok(())
    }

    #[test]
    fn test_overflow() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFFABCD\x80AAABBBB".to_vec();

        let bad_tests = vec![
            // Index  Expected  Definition

            // 8-bit
            (  15,    Some("42"),     (Format::Hex, Size::Eight, Endian::Big) ),
            (  16,    None,           (Format::Hex, Size::Eight, Endian::Big) ),

            // 16-bit
            (  14,    Some("4242"),   (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  15,    None,           (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  16,    None,           (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  14,    Some("4242"),   (Format::Hex, Size::Sixteen, Endian::Little) ),
            (  15,    None,           (Format::Hex, Size::Sixteen, Endian::Little) ),
            (  16,    None,           (Format::Hex, Size::Sixteen, Endian::Little) ),

            // 32-bit
            (  12,    Some("42424242"),   (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  13,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  14,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  15,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  16,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  12,    Some("42424242"),   (Format::Hex, Size::ThirtyTwo, Endian::Little) ),
            (  13,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Little) ),
            (  14,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Little) ),
            (  15,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Little) ),
            (  16,    None,               (Format::Hex, Size::ThirtyTwo, Endian::Little) ),

            // 64-bit
            (  8,    Some("8041414142424242"), (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  9,    None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  10,   None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  11,   None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  12,   None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  13,   None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  14,   None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  15,   None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  16,   None,                     (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  8,    Some("4242424241414180"), (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  9,    None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  10,   None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  11,   None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  12,   None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  13,   None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  14,   None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  15,   None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  16,   None,                     (Format::Hex, Size::SixtyFour, Endian::Little) ),
        ];

        for (index, expected, definition) in bad_tests {
            let d = NumberDefinition::from(definition);

            match expected {
                // Some = a value is there
                Some(e) => assert_eq!(e.to_string(), d.to_string(&(&data, index).into())?),

                // None = we expect an error
                None => assert!(d.to_string(&(&data, index).into()).is_err()),
            }
        }

        Ok(())
    }

    #[test]
    fn test_short_data() -> SimpleResult<()> {
        let data = b"A".to_vec();

        let bad_tests = vec![
            // Index  Expected  Definition

            // 8-bit
            (  0,    Some("41"),     (Format::Hex, Size::Eight, Endian::Big) ),
            (  1,    None,           (Format::Hex, Size::Eight, Endian::Big) ),

            // 16-bit
            (  0,    None,           (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  1,    None,           (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  0,    None,           (Format::Hex, Size::Sixteen, Endian::Little) ),
            (  1,    None,           (Format::Hex, Size::Sixteen, Endian::Little) ),

            // 32-bit
            (  0,    None,           (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  1,    None,           (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  0,    None,           (Format::Hex, Size::ThirtyTwo, Endian::Little) ),
            (  1,    None,           (Format::Hex, Size::ThirtyTwo, Endian::Little) ),

            // 64-bit
            (  0,    None,           (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  1,    None,           (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  0,    None,           (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  1,    None,           (Format::Hex, Size::SixtyFour, Endian::Little) ),
        ];

        for (index, expected, definition) in bad_tests {
            let d = NumberDefinition::from(definition);

            match expected {
                // Some = a value is there
                Some(e) => assert_eq!(e.to_string(), d.to_string(&(&data, index).into())?),

                // None = we expect an error
                None => assert!(d.to_string(&(&data, index).into()).is_err()),
            }
        }

        Ok(())
    }

    #[test]
    fn test_no_data() -> SimpleResult<()> {
        let data = b"".to_vec();

        let bad_tests = vec![
            // Index  Expected  Definition

            // 8-bit
            (  0,    (Format::Hex, Size::Eight, Endian::Big) ),
            (  1,    (Format::Hex, Size::Eight, Endian::Big) ),

            // 16-bit
            (  0,    (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  1,    (Format::Hex, Size::Sixteen, Endian::Big) ),
            (  0,    (Format::Hex, Size::Sixteen, Endian::Little) ),
            (  1,    (Format::Hex, Size::Sixteen, Endian::Little) ),

            // 32-bit
            (  0,    (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  1,    (Format::Hex, Size::ThirtyTwo, Endian::Big) ),
            (  0,    (Format::Hex, Size::ThirtyTwo, Endian::Little) ),
            (  1,    (Format::Hex, Size::ThirtyTwo, Endian::Little) ),

            // 64-bit
            (  0,    (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  1,    (Format::Hex, Size::SixtyFour, Endian::Big) ),
            (  0,    (Format::Hex, Size::SixtyFour, Endian::Little) ),
            (  1,    (Format::Hex, Size::SixtyFour, Endian::Little) ),
        ];

        for (index, definition) in bad_tests {
            let d = NumberDefinition::from(definition);

            // We ALWAYS expect an error
            assert!(d.to_string(&(&data, index).into()).is_err());
        }

        Ok(())
    }
}
