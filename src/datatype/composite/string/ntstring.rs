#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use simple_error::{bail, SimpleResult};
use std::iter::FromIterator;

use crate::datatype::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};
use crate::datatype::composite::H2Array;

/// Defines a null-terminated string.
///
/// This is a string with a NUL byte at the end (`'\0'`). The character type can
/// be any type defined in [`crate::datatype::simple::Character`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct NTString {
    character: Box<H2Type>,
}

impl NTString {
    pub fn new_aligned(alignment: Alignment, character: H2Type) -> SimpleResult<H2Type> {
        if !character.can_be_char() {
            bail!("Character type can't become a character");
        }

        Ok(H2Type::new(alignment, H2Types::NTString(Self {
            character: Box::new(character),
        })))
    }

    pub fn new(character: H2Type) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, character)
    }

    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<char>)> {
        let mut position = offset.position();
        let mut result = Vec::new();

        loop {
            let this_offset = offset.at(position);
            let this_size = self.character.aligned_size(this_offset)?;
            let this_character = self.character.to_char(this_offset)?;

            result.push(this_character);
            position = position + this_size;

            if this_character == '\0' {
                break;
            }
        }

        Ok((position - offset.position(), result))
    }
}

impl H2TypeTrait for NTString {
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
        Ok(String::from_iter(chars[0..(chars.len() - 1)].into_iter()))
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        Ok(format!("\"{}\"", self.to_string(offset)?))
    }

    fn children(&self, offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        // We want the number of characters, not the length in bytes
        let (_, characters) = self.analyze(offset)?;

        Ok(vec![
            (None, H2Array::new(characters.len() as u64, self.character.as_ref().clone())?)
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use crate::sized_number::{Context, Endian};
    use crate::datatype::simple::network::IPv4;
    use crate::datatype::simple::character::{UTF8, ASCII, StrictASCII};
    use crate::datatype::Alignment;

    #[test]
    fn test_utf8_string() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(UTF8::new())?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_string() -> SimpleResult<()> {
        let data = b"\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(UTF8::new())?;
        assert_eq!("\"\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_blank_string() -> SimpleResult<()> {
        let data = b"".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(UTF8::new())?;
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_missing_terminator() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(UTF8::new())?;
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_aligned_characters_string() -> SimpleResult<()> {
        // We're aligning to 3-byte characters, so 1, 2, and 4 byte characters
        // get padded
        //             --    --    ----------  ----------  --------------    --------------    ------
        let data = b"\x41PP\x42PP\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9EPP\xF0\x9F\x98\x88PP\xc3\xb7P\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(UTF8::new_aligned(Alignment::Loose(3)))?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a: H2Type = NTString::new(UTF8::new())?;
        let array = a.resolve(offset, None)?;

        // Should just have one child - the array
        assert_eq!(1, array.children.len());
        assert_eq!("AB❄☢𝄞😈÷", array.as_string.unwrap());
        assert_eq!("\"AB❄☢𝄞😈÷\"", array.display);

        // The child should be an array of the characters, including the NUL at
        // the end
        assert_eq!("[ 'A', 'B', '❄', '☢', '𝄞', '😈', '÷', '\\0' ]", array.children[0].display);
        assert_eq!(8, array.children[0].children.len());

        Ok(())
    }

    #[test]
    fn test_bad_character_type() -> SimpleResult<()> {
        assert!(NTString::new(IPv4::new(Endian::Big)).is_err());
        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"hi\0bye\0test\0".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Array::new(3, NTString::new(
          ASCII::new(StrictASCII::Strict),
        )?)?;

        assert_eq!(12, t.actual_size(offset).unwrap());

        assert_eq!("[ \"hi\", \"bye\", \"test\" ]", t.to_display(offset).unwrap());

        Ok(())
    }
}
