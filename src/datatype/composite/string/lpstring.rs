use serde::{Serialize, Deserialize};

use simple_error::{bail, SimpleResult};
use std::iter::FromIterator;

use crate::datatype::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};
use crate::datatype::composite::H2Array;

/// Defines a length-prefixed string.
///
/// This is a string with a numerical prefix that denotes the length of the
/// string (in *characters*). The length is any numerical value as defined in
/// [`crate::generic_number::GenericReader`] that `can_be_u64()`, and the
/// character type is from the same module, but `can_be_char()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPString {
    length: Box<H2Type>,
    character: Box<H2Type>,
}

impl LPString {
    pub fn new_aligned(alignment: Alignment, length: H2Type, character: H2Type) -> SimpleResult<H2Type> {
        if !length.can_be_number() {
            bail!("Length type isn't numeric!");
        }

        if !character.can_be_number() {
            bail!("Character type can't become a character");
        }

        Ok(H2Type::new(alignment, H2Types::LPString(Self {
            length: Box::new(length),
            character: Box::new(character),
        })))
    }

    pub fn new(length: H2Type, character: H2Type) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, length, character)
    }

    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<char>)> {
        // TODO: This should be usize
        let length = self.length.to_number(offset)?.as_u64()?;

        let mut position = offset.position() + self.length.aligned_size(offset)?;

        let mut result = Vec::new();
        for _ in 0..length {
            let this_offset = offset.at(position);
            let this_size = self.character.actual_size(this_offset)?;
            let this_character = self.character.to_number(this_offset)?.as_char()?;

            result.push(this_character);
            position = position + this_size;
        }

        Ok((position - offset.position(), result))
    }
}

impl H2TypeTrait for LPString {
    fn is_static(&self) -> bool {
        self.character.is_static()
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        Ok(self.analyze(offset)?.0)
    }

    fn can_be_string(&self) -> bool {
        true
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        // Get the length so we can truncate
        let (_, chars) = self.analyze(offset)?;

        // Convert into a string
        Ok(String::from_iter(chars.into_iter()))
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        Ok(format!("\"{}\"", self.to_string(offset)?))
    }

    fn children(&self, offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        let length = self.length.to_number(offset)?.as_u64()?;

        Ok(vec![
            // The size field
            ( Some("size".to_string()), self.length.as_ref().clone() ),

            // The data field
            ( None,                     H2Array::new(length, self.character.as_ref().clone())? ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use crate::generic_number::{Context, GenericReader, Endian, DefaultFormatter, HexFormatter};
    use crate::datatype::simple::H2Number;
    use crate::datatype::simple::network::IPv4;

    #[test]
    fn test_utf8_lpstring() -> SimpleResult<()> {
        //                     --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x00\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));


        let a = LPString::new(
            H2Number::new(GenericReader::U16(Endian::Big), DefaultFormatter::new()),
            H2Number::new_utf8(),
        )?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_lpstring() -> SimpleResult<()> {
        let data = b"\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new_utf8(),
        )?;
        assert_eq!("\"\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_blank_lpstring() -> SimpleResult<()> {
        let data = b"".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new_utf8(),
        )?;
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_aligned_length_lpstring() -> SimpleResult<()> {
        let data = b"\x00\x07PPPPPP\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));


        let a = LPString::new(
            H2Number::new_aligned(Alignment::Loose(8), GenericReader::U16(Endian::Big), DefaultFormatter::new()),
            H2Number::new_utf8(),
        )?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //                 --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a: H2Type = LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new_utf8(),
        )?;
        let array = a.resolve(offset, None)?;

        // Should just have two children - the length and the array
        assert_eq!(2, array.children.len());

        // The first child should just be the length
        assert_eq!("7", array.children[0].display);

        // The second child should be an array of the characters
        assert_eq!("[ 'A', 'B', '❄', '☢', '𝄞', '😈', '÷' ]", array.children[1].display);
        assert_eq!(7, array.children[1].children.len());

        Ok(())
    }

    #[test]
    fn test_bad_type() -> SimpleResult<()> {
        let size_type = H2Number::new(GenericReader::U8, DefaultFormatter::new());
        assert!(LPString::new(size_type, IPv4::new(Endian::Big)).is_err());

        let size_type = IPv4::new(Endian::Big);
        assert!(
            LPString::new(
                size_type,
                H2Number::new_utf8(),
            ).is_err()
        );

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"\x02hi\x03bye\x04test".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Array::new(3, LPString::new(
          H2Number::new(GenericReader::U8, HexFormatter::pretty()),
          H2Number::new_ascii(),
        )?)?;

        assert_eq!(12, t.actual_size(offset)?);
        assert_eq!("[ \"hi\", \"bye\", \"test\" ]", t.to_display(offset)?);

        Ok(())
    }
}
