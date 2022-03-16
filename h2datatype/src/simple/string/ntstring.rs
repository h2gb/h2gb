use std::iter::FromIterator;
use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use generic_number::{Context, Character, CharacterReader, CharacterRenderer};

use crate::{H2Type, H2Types, H2TypeTrait, Alignment, Data};

/// Defines a null-terminated string.
///
/// This is a string with a NUL byte at the end (`'\0'`). The character type can
/// be any type defined in [`generic_number::CharacterReader`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NTString {
    character: CharacterReader,
    renderer: CharacterRenderer,
}

impl NTString {
    pub fn new_aligned(alignment: Alignment, character: impl Into<CharacterReader>, renderer: impl Into<CharacterRenderer>) -> H2Type {
        H2Type::new(alignment, H2Types::NTString(Self {
            character: character.into(),
            renderer: renderer.into(),
        }))
    }

    pub fn new(character: impl Into<CharacterReader>, renderer: impl Into<CharacterRenderer>) -> H2Type {
        Self::new_aligned(Alignment::None, character, renderer)
    }

    fn analyze(&self, offset: Context) -> SimpleResult<(usize, Vec<Character>)> {
        let mut position = offset.position();
        let mut result = Vec::new();

        loop {
            let this_character = self.character.read(offset.at(position))?;
            position = position + this_character.size();

            if this_character.as_char() == '\0' {
                break;
            }

            // Add the result after breaking, so we don't include the '\0'
            result.push(this_character);
        }

        Ok((position - offset.position(), result))
    }
}

impl H2TypeTrait for NTString {
    fn base_size(&self, offset: Context) -> SimpleResult<usize> {
        Ok(self.analyze(offset)?.0)
    }

    fn can_be_string(&self) -> bool {
        true
    }

    fn to_string(&self, offset: Context, _data: &Data) -> SimpleResult<String> {
        // Get the length so we can truncate
        let (_, chars) = self.analyze(offset)?;

        // Convert into a string
        Ok(String::from_iter(chars.into_iter().map(|c| self.renderer.render_character(c))))
    }

    fn to_display(&self, offset: Context, data: &Data) -> SimpleResult<String> {
        Ok(format!("\"{}\"", self.to_string(offset, data)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, CharacterReader, CharacterFormatter};
    use pretty_assertions::assert_eq;

    use crate::composite::H2Array;

    #[test]
    fn test_utf8_string() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Context::new(&data);

        let a = NTString::new(CharacterReader::UTF8, CharacterFormatter::new_pretty_str());
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_string() -> SimpleResult<()> {
        let data = b"\x00".to_vec();
        let offset = Context::new(&data);

        let a = NTString::new(CharacterReader::UTF8, CharacterFormatter::new_pretty_str());
        assert_eq!("\"\"", a.to_display(offset, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_blank_string() -> SimpleResult<()> {
        let data = b"".to_vec();
        let offset = Context::new(&data);

        let a = NTString::new(CharacterReader::UTF8, CharacterFormatter::new_pretty_str());
        assert!(a.to_display(offset, &Data::default()).is_err());

        Ok(())
    }

    #[test]
    fn test_missing_terminator() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Context::new(&data);

        let a = NTString::new(CharacterReader::UTF8, CharacterFormatter::new_pretty_str());
        assert!(a.to_display(offset, &Data::default()).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Context::new(&data);

        let a: H2Type = NTString::new(CharacterReader::UTF8, CharacterFormatter::new_pretty_str());
        let array = a.resolve(offset, None, &Data::default())?;

        // Should just have one child - the array
        assert_eq!("AB❄☢𝄞😈÷", array.as_string.unwrap());
        assert_eq!("\"AB❄☢𝄞😈÷\"", array.display);

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"hi\0bye\0test\0".to_vec();
        let offset = Context::new(&data);

        let t = H2Array::new(3, NTString::new(
            CharacterReader::ASCII,
            CharacterFormatter::new_pretty_str(),
        ))?;

        assert_eq!(12, t.base_size(offset).unwrap());

        assert_eq!("[ \"hi\", \"bye\", \"test\" ]", t.to_display(offset, &Data::default()).unwrap());

        Ok(())
    }
}
