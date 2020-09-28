use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum NumberFormat {
    Hex,
    HexUppercase,
    DecimalSigned,
    DecimalUnsigned,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum NumberSize {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl NumberSize {
    pub fn len(self) -> usize {
        match self {
            Self::Eight => 1,
            Self::Sixteen => 2,
            Self::ThirtyTwo => 4,
            Self::SixtyFour => 8,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NumberDefinition {
    format: NumberFormat,
    size: NumberSize,
    endian: Endian,
}

impl From<(NumberFormat, NumberSize, Endian)> for NumberDefinition {
    fn from(o: (NumberFormat, NumberSize, Endian)) -> NumberDefinition {
        Self {
            format: o.0,
            size: o.1,
            endian: o.2,
        }
    }
}

impl NumberDefinition {
    pub fn new(format: NumberFormat, size: NumberSize, endian: Endian) -> Self {
        Self {
            format: format,
            size:   size,
            endian: endian,
        }
    }

    pub fn u8() -> Self {
        Self::new(NumberFormat::Hex, NumberSize::Eight, Endian::Big)
    }

    pub fn u16_big() -> Self {
        Self::new(NumberFormat::Hex, NumberSize::Sixteen, Endian::Big)
    }

    pub fn u16_little() -> Self {
        Self::new(NumberFormat::Hex, NumberSize::Sixteen, Endian::Little)
    }

    pub fn u32_big() -> Self {
        Self::new(NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big)
    }

    pub fn u32_little() -> Self {
        Self::new(NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little)
    }

    pub fn u64_big() -> Self {
        Self::new(NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big)
    }

    pub fn u64_little() -> Self {
        Self::new(NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little)
    }

    pub fn len(self) -> usize {
        self.size.len()
    }
}

pub struct H2Context<'a> {
    pub data: &'a Vec<u8>,
    pub index: usize,
}

impl<'a> From<(&'a Vec<u8>, usize)> for H2Context<'a> {
    fn from(o: (&'a Vec<u8>, usize)) -> Self {
        Self::new(o.0, o.1)
    }
}

impl<'a> H2Context<'a> {
    fn new(data: &'a Vec<u8>, index: usize) -> Self {
        Self {
            data: data,
            index: index,
        }
    }

    pub fn set_index(&mut self, index: usize) {
        // Should I sanity check?
        self.index = index;
    }

    pub fn increment_index(&mut self, offset: usize) {
        self.set_index(self.index + offset)
    }

    pub fn read_generic(&self, endian: Endian, size: NumberSize) -> SimpleResult<u64> {
        // Get the length and make sure it's actually possible
        if self.index + size.len() > self.data.len() {
            bail!("Overflow");
        }

        let mut n = 0;

        // Loop through as much as we need to
        for i in 0..size.len() {
            match endian {
                // Loop forward through the bytes for big endian
                Endian::Big => {
                    n = (n << 8) | self.data[self.index + i] as u64;
                },

                // Loop backward through the bytes for little endian
                Endian::Little => {
                    n = (n << 8) | self.data[self.index + size.len() - i - 1] as u64;
                },
            };
        }

        Ok(n)
    }

    pub fn consume_generic(&mut self, endian: Endian, size: NumberSize) -> SimpleResult<u64> {
        // If there's an error, short-circuit right here
        let result = self.read_generic(endian, size)?;

        // If that succeeded, increment
        self.increment_index(size.len());

        Ok(result)
    }

    pub fn read_u8(&self) -> SimpleResult<u8> {
        // Endian doesn't matter for 8-bit values
        Ok(self.read_generic(Endian::Big, NumberSize::Eight)? as u8)
    }
    pub fn read_u16(&self, endian: Endian) -> SimpleResult<u16> {
        Ok(self.read_generic(endian, NumberSize::Sixteen)? as u16)
    }
    pub fn read_u32(&self, endian: Endian) -> SimpleResult<u32> {
        Ok(self.read_generic(endian, NumberSize::ThirtyTwo)? as u32)
    }
    pub fn read_u64(&self, endian: Endian) -> SimpleResult<u64> {
        Ok(self.read_generic(endian, NumberSize::SixtyFour)? as u64)
    }

    pub fn read_i8(&self) -> SimpleResult<i8> {
        // Endian doesn't matter for 8-bit values
        Ok(self.read_generic(Endian::Big, NumberSize::Eight)? as i8)
    }
    pub fn read_i16(&self, endian: Endian) -> SimpleResult<i16> {
        Ok(self.read_generic(endian, NumberSize::Sixteen)? as i16)
    }
    pub fn read_i32(&self, endian: Endian) -> SimpleResult<i32> {
        Ok(self.read_generic(endian, NumberSize::ThirtyTwo)? as i32)
    }
    pub fn read_i64(&self, endian: Endian) -> SimpleResult<i64> {
        Ok(self.read_generic(endian, NumberSize::SixtyFour)? as i64)
    }

    pub fn consume_u8(&mut self) -> SimpleResult<u8> {
        // Endian doesn't matter for 8-bit values
        Ok(self.consume_generic(Endian::Big, NumberSize::Eight)? as u8)
    }
    pub fn consume_u16(&mut self, endian: Endian) -> SimpleResult<u16> {
        Ok(self.consume_generic(endian, NumberSize::Sixteen)? as u16)
    }
    pub fn consume_u32(&mut self, endian: Endian) -> SimpleResult<u32> {
        Ok(self.consume_generic(endian, NumberSize::ThirtyTwo)? as u32)
    }
    pub fn consume_u64(&mut self, endian: Endian) -> SimpleResult<u64> {
        Ok(self.consume_generic(endian, NumberSize::SixtyFour)? as u64)
    }

    pub fn consume_i8(&mut self) -> SimpleResult<i8> {
        // Endian doesn't matter for 8-bit values
        Ok(self.consume_generic(Endian::Big, NumberSize::Eight)? as i8)
    }
    pub fn consume_i16(&mut self, endian: Endian) -> SimpleResult<i16> {
        Ok(self.consume_generic(endian, NumberSize::Sixteen)? as i16)
    }
    pub fn consume_i32(&mut self, endian: Endian) -> SimpleResult<i32> {
        Ok(self.consume_generic(endian, NumberSize::ThirtyTwo)? as i32)
    }
    pub fn consume_i64(&mut self, endian: Endian) -> SimpleResult<i64> {
        Ok(self.consume_generic(endian, NumberSize::SixtyFour)? as i64)
    }

    pub fn read_number_as_string(&self, definition: NumberDefinition) -> SimpleResult<String> {
        let size   = definition.size;
        let endian = definition.endian;
        let format = definition.format;

        match size {
            NumberSize::Eight => {
                Ok(match format {
                    NumberFormat::Hex             => format!("{:02x}", self.read_u8()?),
                    NumberFormat::HexUppercase    => format!("{:02X}", self.read_u8()?),
                    NumberFormat::DecimalUnsigned => format!("{}",     self.read_u8()?),
                    NumberFormat::DecimalSigned   => format!("{}",     self.read_i8()?),
                }.to_string())
            },
            NumberSize::Sixteen => {
                Ok(match format {
                    NumberFormat::Hex             => format!("{:04x}", self.read_u16(endian)?),
                    NumberFormat::HexUppercase    => format!("{:04X}", self.read_u16(endian)?),
                    NumberFormat::DecimalUnsigned => format!("{}",     self.read_u16(endian)?),
                    NumberFormat::DecimalSigned   => format!("{}",     self.read_i16(endian)?),
                }.to_string())
            },
            NumberSize::ThirtyTwo => {
                Ok(match format {
                    NumberFormat::Hex             => format!("{:08x}", self.read_u32(endian)?),
                    NumberFormat::HexUppercase    => format!("{:08X}", self.read_u32(endian)?),
                    NumberFormat::DecimalUnsigned => format!("{}",     self.read_u32(endian)?),
                    NumberFormat::DecimalSigned   => format!("{}",     self.read_i32(endian)?),
                }.to_string())
            },
            NumberSize::SixtyFour => {
                Ok(match format {
                    NumberFormat::Hex             => format!("{:016x}", self.read_u64(endian)?),
                    NumberFormat::HexUppercase    => format!("{:016X}", self.read_u64(endian)?),
                    NumberFormat::DecimalUnsigned => format!("{}",      self.read_u64(endian)?),
                    NumberFormat::DecimalSigned   => format!("{}",      self.read_i64(endian)?),
                }.to_string())
            },
        }
    }

    pub fn consume_number_as_string(&mut self, definition: NumberDefinition) -> SimpleResult<String> {
        let result = self.read_number_as_string(definition)?;

        self.increment_index(definition.size.len());

        Ok(result)
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
            (  0,     "00",     (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),
            (  1,     "7f",     (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),
            (  2,     "80",     (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),
            (  3,     "ff",     (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),
            (  3,     "FF",     (NumberFormat::HexUppercase, NumberSize::Eight, Endian::Big) ),

            // 8-bit unsigned
            (  0,     "0",      (NumberFormat::DecimalUnsigned, NumberSize::Eight, Endian::Big) ),
            (  1,     "127",    (NumberFormat::DecimalUnsigned, NumberSize::Eight, Endian::Big) ),
            (  2,     "128",    (NumberFormat::DecimalUnsigned, NumberSize::Eight, Endian::Big) ),
            (  3,     "255",    (NumberFormat::DecimalUnsigned, NumberSize::Eight, Endian::Big) ),

            // 8-bit signed
            (  0,     "0",      (NumberFormat::DecimalSigned, NumberSize::Eight, Endian::Big) ),
            (  1,     "127",    (NumberFormat::DecimalSigned, NumberSize::Eight, Endian::Big) ),
            (  2,     "-128",   (NumberFormat::DecimalSigned, NumberSize::Eight, Endian::Big) ),
            (  3,     "-1",     (NumberFormat::DecimalSigned, NumberSize::Eight, Endian::Big) ),

            // 16-bit, big endian, hex
            (  0,     "007f",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  1,     "7f80",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  2,     "80ff",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  3,     "ff41",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  3,     "FF41",   (NumberFormat::HexUppercase, NumberSize::Sixteen, Endian::Big) ),

            // 16-bit, little endian, hex
            (  0,     "7f00",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),
            (  1,     "807f",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),
            (  2,     "ff80",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),
            (  3,     "41ff",   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),

            // 16-bit, big endian, unsigned
            (  0,     "127",    (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Big) ),
            (  1,     "32640",  (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Big) ),
            (  2,     "33023",  (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Big) ),
            (  3,     "65345",  (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Big) ),

            // 16-bit, little endian, unsigned
            (  0,     "32512",  (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Little) ),
            (  1,     "32895",  (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Little) ),
            (  2,     "65408",  (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Little) ),
            (  3,     "16895",  (NumberFormat::DecimalUnsigned, NumberSize::Sixteen, Endian::Little) ),

            // 16-bit, big endian, signed
            (  0,     "127",    (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Big) ),
            (  1,     "32640",  (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Big) ),
            (  2,     "-32513", (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Big) ),
            (  3,     "-191",   (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Big) ),

            // 16-bit, little endian, signed
            (  0,     "32512",   (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Little) ),
            (  1,     "-32641",  (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Little) ),
            (  2,     "-128",    (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Little) ),
            (  3,     "16895",   (NumberFormat::DecimalSigned, NumberSize::Sixteen, Endian::Little) ),

            // 32-bit, big / little endian, hex
            (  0,     "007f80ff", (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  0,     "ff807f00", (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),

            // 32-bit, big / little endian, unsigned
            (  0,     "8356095",    (NumberFormat::DecimalUnsigned, NumberSize::ThirtyTwo, Endian::Big) ),
            (  0,     "4286611200", (NumberFormat::DecimalUnsigned, NumberSize::ThirtyTwo, Endian::Little) ),

            // 32-bit, big / little endian, signed
            (  0,     "8356095",  (NumberFormat::DecimalSigned, NumberSize::ThirtyTwo, Endian::Big) ),
            (  0,     "-8356096", (NumberFormat::DecimalSigned, NumberSize::ThirtyTwo, Endian::Little) ),

            // 64-bit, big / little endian, hex
            (  0,     "007f80ff41424344", (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  8,     "8041414142424242", (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  0,     "44434241ff807f00", (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
            (  8,     "4242424241414180", (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),

            // 64-bit, big / little endian, unsigned
            (  0,     "35889155842130756",   (NumberFormat::DecimalUnsigned, NumberSize::SixtyFour, Endian::Big) ),
            (  8,     "9241739658881286722", (NumberFormat::DecimalUnsigned, NumberSize::SixtyFour, Endian::Big) ),
            (  0,     "4918848069245632256", (NumberFormat::DecimalUnsigned, NumberSize::SixtyFour, Endian::Little) ),
            (  8,     "4774451407296217472", (NumberFormat::DecimalUnsigned, NumberSize::SixtyFour, Endian::Little) ),

            // // 64-bit, big / little endian, signed
            // (  0,     "8356095",  (NumberFormat::DecimalSigned, NumberSize::SixtyFour, Endian::Big) ),
            // (  0,     "-8356096", (NumberFormat::DecimalSigned, NumberSize::SixtyFour, Endian::Little) ),
            (  0,     "35889155842130756",    (NumberFormat::DecimalSigned, NumberSize::SixtyFour, Endian::Big) ),
            (  8,     "-9205004414828264894", (NumberFormat::DecimalSigned, NumberSize::SixtyFour, Endian::Big) ),
            (  0,     "4918848069245632256",  (NumberFormat::DecimalSigned, NumberSize::SixtyFour, Endian::Little) ),
            (  8,     "4774451407296217472",  (NumberFormat::DecimalSigned, NumberSize::SixtyFour, Endian::Little) ),
        ];

        for (index, expected, definition) in tests {
            let c = H2Context::from((&data, index));
            let d = NumberDefinition::from(definition);
            assert_eq!(expected.to_string(), c.read_number_as_string(d)?);
        }

        Ok(())
    }

    #[test]
    fn test_overflow() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFFABCD\x80AAABBBB".to_vec();

        let bad_tests = vec![
            // Index  Expected  Definition

            // 8-bit
            (  15,    Some("42"),     (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),
            (  16,    None,           (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),

            // 16-bit
            (  14,    Some("4242"),   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  15,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  16,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  14,    Some("4242"),   (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),
            (  15,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),
            (  16,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),

            // 32-bit
            (  12,    Some("42424242"),   (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  13,    None,               (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  16,    None,               (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  12,    Some("42424242"),   (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),
            (  13,    None,               (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),
            (  16,    None,               (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),

            // 64-bit
            (  8,    Some("8041414142424242"), (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  9,    None,                     (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  16,   None,                     (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  8,    Some("4242424241414180"), (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
            (  9,    None,                     (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
            (  16,   None,                     (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
        ];

        for (index, expected, definition) in bad_tests {
            let c = H2Context::from((&data, index));
            let d = NumberDefinition::from(definition);

            match expected {
                Some(expected) => assert_eq!(expected.to_string(), c.read_number_as_string(d)?),
                None => assert!(c.read_number_as_string(d).is_err()),
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
            (  0,    Some("41"),     (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),
            (  1,    None,           (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),

            // 16-bit
            (  0,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  1,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  0,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),
            (  1,    None,           (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),

            // 32-bit
            (  0,    None,           (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  1,    None,           (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  0,    None,           (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),
            (  1,    None,           (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),

            // 64-bit
            (  0,    None,           (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  1,    None,           (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  0,    None,           (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
            (  1,    None,           (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
        ];

        for (index, expected, definition) in bad_tests {
            let c = H2Context::from((&data, index));
            let d = NumberDefinition::from(definition);

            match expected {
                Some(expected) => assert_eq!(expected.to_string(), c.read_number_as_string(d)?),
                None => assert!(c.read_number_as_string(d).is_err()),
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
            (  0,    (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),
            (  1,    (NumberFormat::Hex, NumberSize::Eight, Endian::Big) ),

            // 16-bit
            (  0,    (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  1,    (NumberFormat::Hex, NumberSize::Sixteen, Endian::Big) ),
            (  0,    (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),
            (  1,    (NumberFormat::Hex, NumberSize::Sixteen, Endian::Little) ),

            // 32-bit
            (  0,    (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  1,    (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Big) ),
            (  0,    (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),
            (  1,    (NumberFormat::Hex, NumberSize::ThirtyTwo, Endian::Little) ),

            // 64-bit
            (  0,    (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  1,    (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Big) ),
            (  0,    (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
            (  1,    (NumberFormat::Hex, NumberSize::SixtyFour, Endian::Little) ),
        ];

        for (index, definition) in bad_tests {
            let c = H2Context::from((&data, index));
            let d = NumberDefinition::from(definition);

            assert!(c.read_number_as_string(d).is_err());
        }

        Ok(())
    }

    #[test]
    fn test_consume() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFFABCD\x80AAABBBB".to_vec();
        let mut c = H2Context::from((&data, 0));

        assert_eq!(0x00, c.consume_u8()?);
        assert_eq!(0x7f, c.consume_u8()?);
        assert_eq!(0x80, c.consume_u8()?);
        assert_eq!(0xff, c.consume_u8()?);

        c.set_index(0);
        assert_eq!(0x007f, c.consume_u16(Endian::Big)?);
        assert_eq!(0x80ff4142, c.consume_u32(Endian::Big)?);
        assert_eq!(0x4344804141414242, c.consume_u64(Endian::Big)?);

        c.set_index(0);
        assert_eq!(0x7f00, c.consume_u16(Endian::Little)?);
        assert_eq!(0x4241ff80, c.consume_u32(Endian::Little)?);
        assert_eq!(0x4242414141804443, c.consume_u64(Endian::Little)?);

        Ok(())
    }
}
