use serde::{Serialize, Deserialize};

use simple_error::{bail, SimpleResult};
use std::iter::FromIterator;

use crate::datatype::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};
use crate::datatype::composite::H2Array;

/// Defines a string with a configured length.
///
/// The length (in characters) is chosen when creating the type. The length in
/// bytes may be longer if the character type is non-ASCII, however. See
/// [`crate::datatype::simple::Character`] for a list of possible character types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2String {
    length: u64,
    character: Box<H2Type>,
}

impl H2String {
    pub fn new_aligned(alignment: Alignment, length_in_characters: u64, character: H2Type) -> SimpleResult<H2Type> {
        if length_in_characters == 0 {
            bail!("Length must be at least 1 character long");
        }

        if !character.can_be_char() {
            bail!("Character type can't become a character");
        }

        Ok(H2Type::new(alignment, H2Types::H2String(Self {
            length: length_in_characters,
            character: Box::new(character),
        })))
    }

    pub fn new(length_in_characters: u64, character: H2Type) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, length_in_characters, character)
    }


    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<char>)> {
        let mut position = offset.position();
        let mut result = Vec::new();

        for _ in 0..self.length {
            let this_offset = offset.at(position);
            let this_size = self.character.actual_size(this_offset)?;
            let this_character = self.character.to_char(this_offset)?;

            result.push(this_character);
            position = position + this_size;
        }

        Ok((position - offset.position(), result))
    }
}

impl H2TypeTrait for H2String {
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

    fn children(&self, _offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        Ok(vec![
            ( None, H2Array::new(self.length, self.character.as_ref().clone())? ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use crate::generic_number::{Context, Endian};
    use crate::datatype::simple::network::IPv4;
    use crate::datatype::simple::character::{UTF8, ASCII, StrictASCII};

    #[test]
    fn test_utf8_lstring() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = H2String::new(7, UTF8::new())?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_lstring() -> SimpleResult<()> {
        assert!(H2String::new(0, UTF8::new()).is_err());

        Ok(())
    }

    #[test]
    fn test_too_long_lstring() -> SimpleResult<()> {
        let data = b"A".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = H2String::new(2, UTF8::new())?;
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a: H2Type = H2String::new(7, UTF8::new())?;
        let array = a.resolve(offset, None)?;

        // Should just have one child - the array
        assert_eq!(1, array.children.len());

        // The child should be an array of the characters
        assert_eq!("[ 'A', 'B', '❄', '☢', '𝄞', '😈', '÷' ]", array.children[0].display);
        assert_eq!(7, array.children[0].children.len());

        Ok(())
    }

    #[test]
    fn test_bad_type() -> SimpleResult<()> {
        assert!(H2String::new(1, IPv4::new(Endian::Big)).is_err());
        assert!(H2String::new(0, UTF8::new()).is_err());

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Array::new(4, H2String::new(4,
          ASCII::new(StrictASCII::Strict),
        )?)?;

        assert_eq!(16, t.actual_size(offset).unwrap());

        assert_eq!("[ \"AAAA\", \"BBBB\", \"CCCC\", \"DDDD\" ]", t.to_display(offset).unwrap());

        Ok(())
    }
}
