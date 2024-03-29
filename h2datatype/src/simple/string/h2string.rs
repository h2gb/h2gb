use std::iter::FromIterator;

use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::{Character, CharacterReader, CharacterRenderer};

use crate::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};

/// Defines a string with a configured length.
///
/// The length (in characters) is chosen when creating the type. The length in
/// bytes may be longer if the character type is non-ASCII, however. See
/// [`generic_number::CharacterReader`] for a list of possible character
/// types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2String {
    length: u64,
    character: CharacterReader,
    renderer: CharacterRenderer,
}

impl H2String {
    pub fn new_aligned(alignment: Alignment, length_in_characters: u64, character: CharacterReader, renderer: CharacterRenderer) -> SimpleResult<H2Type> {
        if length_in_characters == 0 {
            bail!("Length must be at least 1 character long");
        }

        Ok(H2Type::new(alignment, H2Types::H2String(Self {
            length: length_in_characters,
            character: character,
            renderer: renderer,
        })))
    }

    pub fn new(length_in_characters: u64, character: CharacterReader, renderer: CharacterRenderer) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, length_in_characters, character, renderer)
    }


    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<Character>)> {
        let mut position = offset.position();
        let mut result = Vec::new();

        for _ in 0..self.length {
            let this_offset = offset.at(position);

            let this_character = self.character.read(this_offset.get_dynamic()?)?;

            result.push(this_character);
            position = position + this_character.size() as u64;
        }

        Ok((position - offset.position(), result))
    }
}

impl H2TypeTrait for H2String {
    fn is_static(&self) -> bool {
        self.character.size().is_some()
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        match self.character.size() {
            Some(s) => Ok(s as u64 * self.length),
            None => Ok(self.analyze(offset)?.0),
        }
    }

    fn can_be_string(&self) -> bool {
        true
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        // Get the length so we can truncate
        let (_, chars) = self.analyze(offset)?;

        // Convert into a string
        Ok(String::from_iter(chars.into_iter().map(|c| self.renderer.render(c))))
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        Ok(format!("\"{}\"", self.to_string(offset)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, CharacterFormatter, CharacterReplacementPolicy, CharacterUnprintableOption};

    use crate::composite::H2Array;

    #[test]
    fn test_utf8_lstring() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = H2String::new(7, CharacterReader::UTF8, CharacterFormatter::pretty_str_character())?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_lstring() -> SimpleResult<()> {
        assert!(H2String::new(0, CharacterReader::UTF8, CharacterFormatter::pretty_str_character()).is_err());

        Ok(())
    }

    #[test]
    fn test_too_long_lstring() -> SimpleResult<()> {
        let data = b"A".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = H2String::new(2, CharacterReader::UTF8, CharacterFormatter::pretty_str_character())?;
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a: H2Type = H2String::new(7, CharacterReader::UTF8, CharacterFormatter::pretty_str_character())?;
        let resolved = a.resolve(offset, None)?;

        assert_eq!("\"AB❄☢𝄞😈÷\"", resolved.display);

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Array::new(4, H2String::new(4, CharacterReader::ASCII, CharacterFormatter::pretty_str_character())?)?;

        assert_eq!(16, t.actual_size(offset).unwrap());
        assert_eq!("[ \"AAAA\", \"BBBB\", \"CCCC\", \"DDDD\" ]", t.to_display(offset).unwrap());

        Ok(())
    }

    #[test]
    fn test_character_renderer() -> SimpleResult<()> {
        let data = b"\x41\x10\x09".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = H2String::new(3, CharacterReader::ASCII, CharacterFormatter::pretty_str_character())?;
        assert_eq!("\"A\\x10\\t\"", a.to_display(offset)?);

        let a = H2String::new(3, CharacterReader::ASCII, CharacterFormatter::new_character(
                false, // show_single_quotes
                CharacterReplacementPolicy::ReplaceEverything,
                CharacterUnprintableOption::URLEncode,

        ))?;
        assert_eq!("\"%41%10%09\"", a.to_display(offset)?);

        Ok(())
    }
}
