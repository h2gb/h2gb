use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use crate::sized_number::Endian;

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};
use crate::datatype::simple::character::common;

/// Defines a UTF-32 value.
///
/// A UTF-32 value is always 32 bits long.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTF32 {
    endian: Endian,
}

impl UTF32 {
    pub fn new_aligned(alignment: Alignment, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::UTF32(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(Alignment::None, endian)
    }
}

impl H2TypeTrait for UTF32 {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(4)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        Ok(common::display_character(self.to_char(offset)?))
    }

    fn can_be_char(&self) -> bool {
        true
    }

    fn to_char(&self, offset: Offset) -> SimpleResult<char> {
        offset.get_dynamic()?.read_utf32(self.endian)
    }

    fn can_be_u64(&self) -> bool {
        true
    }

    fn to_u64(&self, offset: Offset) -> SimpleResult<u64> {
        Ok(self.to_char(offset)? as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use crate::sized_number::Context;

    #[test]
    fn test_utf32_to_display_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x41\x00\x00\x00\x42\x00\x00\x27\x44\x00\x00\x26\x22\x00\x01\xD1\x1E\x00\x01\xF6\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("'A'", UTF32::new(Endian::Big).to_display(offset.at(0))?);
        assert_eq!("'B'", UTF32::new(Endian::Big).to_display(offset.at(4))?);
        assert_eq!("'❄'", UTF32::new(Endian::Big).to_display(offset.at(8))?);
        assert_eq!("'☢'", UTF32::new(Endian::Big).to_display(offset.at(12))?);
        assert_eq!("'𝄞'", UTF32::new(Endian::Big).to_display(offset.at(16))?);
        assert_eq!("'😈'", UTF32::new(Endian::Big).to_display(offset.at(20))?);

        Ok(())
    }

    #[test]
    fn test_utf32_to_display_little_endian() -> SimpleResult<()> {
        let data = b"\x41\x00\x00\x00\x42\x00\x00\x00\x44\x27\x00\x00\x22\x26\x00\x00\x1E\xd1\x01\x00\x08\xf6\x01\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("'A'", UTF32::new(Endian::Little).to_display(offset.at(0))?);
        assert_eq!("'B'", UTF32::new(Endian::Little).to_display(offset.at(4))?);
        assert_eq!("'❄'", UTF32::new(Endian::Little).to_display(offset.at(8))?);
        assert_eq!("'☢'", UTF32::new(Endian::Little).to_display(offset.at(12))?);
        assert_eq!("'𝄞'", UTF32::new(Endian::Little).to_display(offset.at(16))?);
        assert_eq!("'😈'", UTF32::new(Endian::Little).to_display(offset.at(20))?);

        Ok(())
    }

    #[test]
    fn test_null() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x41\x00\x00\x00\x00\x00\x00\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  UTF32::new(Endian::Big).to_char(offset.at(0))?);
        assert_eq!('\0', UTF32::new(Endian::Big).to_char(offset.at(4))?);
        assert_eq!('A',  UTF32::new(Endian::Big).to_char(offset.at(8))?);

        Ok(())
    }
}
