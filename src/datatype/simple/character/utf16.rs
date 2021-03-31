use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use crate::sized_number::{Endian, Context};

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};
use crate::datatype::simple::character::common;

/// Defines a UTF-16 character.
///
/// UTF-16 characters are two or four bytes long.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTF16 {
    endian: Endian,
}

impl UTF16 {
    pub fn new_aligned(alignment: Alignment, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::UTF16(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(Alignment::None, endian)
    }

    fn read_utf16(context: Context, endian: Endian) -> SimpleResult<(u64, char)> {
        let (size, c) = context.read_utf16(endian)?;

        Ok((size as u64, c))
    }
}

impl H2TypeTrait for UTF16 {
    fn is_static(&self) -> bool {
        false
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        Ok(Self::read_utf16(offset.get_dynamic()?, self.endian)?.0)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        Ok(common::display_character(self.to_char(offset)?))
    }

    fn can_be_char(&self) -> bool {
        true
    }

    fn to_char(&self, offset: Offset) -> SimpleResult<char> {
        let context = offset.get_dynamic()?;

        Ok(Self::read_utf16(context, self.endian)?.1)
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
    fn test_utf16_size_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(0))?);
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(2))?);
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(4))?);
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(6))?);

        // Double
        assert_eq!(4, UTF16::new(Endian::Big).actual_size(offset.at(8))?);
        assert_eq!(4, UTF16::new(Endian::Big).actual_size(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_utf16_to_display_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("'A'", UTF16::new(Endian::Big).to_display(offset.at(0))?);
        assert_eq!("'B'", UTF16::new(Endian::Big).to_display(offset.at(2))?);
        assert_eq!("'❄'", UTF16::new(Endian::Big).to_display(offset.at(4))?);
        assert_eq!("'☢'", UTF16::new(Endian::Big).to_display(offset.at(6))?);

        // Double
        assert_eq!("'𝄞'", UTF16::new(Endian::Big).to_display(offset.at(8))?);
        assert_eq!("'😈'", UTF16::new(Endian::Big).to_display(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_utf16_to_display_little_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x41\x00\x42\x00\x44\x27\x22\x26\x34\xd8\x1e\xdd\x3d\xd8\x08\xde".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("'A'", UTF16::new(Endian::Little).to_display(offset.at(0))?);
        assert_eq!("'B'", UTF16::new(Endian::Little).to_display(offset.at(2))?);
        assert_eq!("'❄'", UTF16::new(Endian::Little).to_display(offset.at(4))?);
        assert_eq!("'☢'", UTF16::new(Endian::Little).to_display(offset.at(6))?);

        // Double
        assert_eq!("'𝄞'", UTF16::new(Endian::Little).to_display(offset.at(8))?);
        assert_eq!("'😈'", UTF16::new(Endian::Little).to_display(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_null() -> SimpleResult<()> {
        let data = b"\x00\x41\x00\x00\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  UTF16::new(Endian::Big).to_char(offset.at(0))?);
        assert_eq!('\0', UTF16::new(Endian::Big).to_char(offset.at(2))?);
        assert_eq!('A',  UTF16::new(Endian::Big).to_char(offset.at(4))?);

        Ok(())
    }
}
