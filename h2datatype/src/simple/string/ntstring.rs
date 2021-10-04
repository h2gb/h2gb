use std::iter::FromIterator;
use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::CharacterReader;

use crate::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};
use crate::composite::H2Array;

/// Defines a null-terminated string.
///
/// This is a string with a NUL byte at the end (`'\0'`). The character type can
/// be any type defined in [`generic_number::GenericReader`] that can
/// be a character (`can_be_char()` is `true`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NTString {
    character: CharacterReader,
}

impl NTString {
    pub fn new_aligned(alignment: Alignment, character: CharacterReader) -> H2Type {
        H2Type::new(alignment, H2Types::NTString(Self {
            character: character,
        }))
    }

    pub fn new(character: CharacterReader) -> H2Type {
        Self::new_aligned(Alignment::None, character)
    }

    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<char>)> {
        let mut position = offset.position();
        let mut result = Vec::new();

        loop {
            let this_character = self.character.read(offset.at(position).get_dynamic()?)?;
            result.push(this_character.as_char());
            position = position + this_character.size() as u64;

            if this_character.as_char() == '\0' {
                break;
            }
        }

        Ok((position - offset.position(), result))
    }
}

impl H2TypeTrait for NTString {
    fn is_static(&self) -> bool {
        self.character.size().is_some()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{DefaultFormatter, Context, Endian, CharacterReader};
    use crate::Alignment;
    use crate::simple::numeric::H2Character;
    use crate::simple::network::IPv4;

    #[test]
    fn test_utf8_string() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(CharacterReader::UTF8);
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_string() -> SimpleResult<()> {
        let data = b"\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(CharacterReader::UTF8);
        assert_eq!("\"\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_blank_string() -> SimpleResult<()> {
        let data = b"".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(CharacterReader::UTF8);
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_missing_terminator() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(CharacterReader::UTF8);
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a: H2Type = NTString::new(CharacterReader::UTF8);
        let array = a.resolve(offset, None)?;

        // Should just have one child - the array
        assert_eq!("AB❄☢𝄞😈÷", array.as_string.unwrap());
        assert_eq!("\"AB❄☢𝄞😈÷\"", array.display);

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"hi\0bye\0test\0".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Array::new(3, NTString::new(
            CharacterReader::ASCII,
        ))?;

        assert_eq!(12, t.actual_size(offset).unwrap());

        assert_eq!("[ \"hi\", \"bye\", \"test\" ]", t.to_display(offset).unwrap());

        Ok(())
    }
}
