#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use sized_number::Context;

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};
use crate::datatype::simple::character::common;

/// Defines a UTF-8 value.
///
/// A UTF-8 value is a character that's represented by 1-4 bytes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct UTF8 {
}

impl UTF8 {
    pub fn new_aligned(alignment: Alignment) -> H2Type {
        H2Type::new(alignment, H2Types::UTF8(Self {
        }))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(Alignment::None)
    }

    fn read_utf8(context: Context) -> SimpleResult<(u64, char)> {
        let (size, c) = context.read_utf8()?;

        Ok((size as u64, c))
    }
}

impl H2TypeTrait for UTF8 {
    fn is_static(&self) -> bool {
        false
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        Ok(Self::read_utf8(offset.get_dynamic()?)?.0)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        Ok(common::display_character(self.to_char(offset)?))
    }

    fn can_be_char(&self) -> bool {
        true
    }

    fn to_char(&self, offset: Offset) -> SimpleResult<char> {
        Ok(Self::read_utf8(offset.get_dynamic()?)?.1)
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
    use sized_number::Context;

    #[test]
    fn test_utf8_size() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!(1, UTF8::new().actual_size(offset.at(0))?);
        assert_eq!(1, UTF8::new().actual_size(offset.at(1))?);
        assert_eq!(3, UTF8::new().actual_size(offset.at(2))?);
        assert_eq!(3, UTF8::new().actual_size(offset.at(5))?);
        assert_eq!(4, UTF8::new().actual_size(offset.at(8))?);
        assert_eq!(4, UTF8::new().actual_size(offset.at(12))?);
        assert_eq!(2, UTF8::new().actual_size(offset.at(16))?);

        Ok(())
    }

    #[test]
    fn test_utf8_to_display() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("'A'", UTF8::new().to_display(offset.at(0))?);
        assert_eq!("'B'", UTF8::new().to_display(offset.at(1))?);
        assert_eq!("'❄'", UTF8::new().to_display(offset.at(2))?);
        assert_eq!("'☢'", UTF8::new().to_display(offset.at(5))?);
        assert_eq!("'𝄞'", UTF8::new().to_display(offset.at(8))?);
        assert_eq!("'😈'", UTF8::new().to_display(offset.at(12))?);
        assert_eq!("'÷'", UTF8::new().to_display(offset.at(16))?);

        Ok(())
    }

    #[test]
    fn test_utf8_too_short() -> SimpleResult<()> {
        let data = b"\xE2".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert!(UTF8::new().to_display(offset.at(0)).is_err());
        assert!(UTF8::new().to_display(offset.at(1)).is_err());

        Ok(())
    }

    #[test]
    fn test_null() -> SimpleResult<()> {
        let data = b"\x41\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  UTF8::new().to_char(offset.at(0))?);
        assert_eq!('\0', UTF8::new().to_char(offset.at(1))?);
        assert_eq!('A',  UTF8::new().to_char(offset.at(2))?);

        Ok(())
    }
}
