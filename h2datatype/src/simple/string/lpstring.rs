use std::iter::FromIterator;
use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::{IntegerReader, Character, CharacterReader, CharacterRenderer};

use crate::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};

/// Defines a length-prefixed string.
///
/// This is a string with a numerical prefix that denotes the length of the
/// string (in *characters*). The length is any numerical value as defined in
/// [`generic_number::IntegerReader`] that `can_be_u64()`, and the
/// character type is from [`generic_number::CharacterReader`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPString {
    length: IntegerReader,
    character: CharacterReader,
    renderer: CharacterRenderer,
}

impl LPString {
    pub fn new_aligned(alignment: Alignment, length: IntegerReader, character: CharacterReader, renderer: CharacterRenderer) -> SimpleResult<H2Type> {
        if !length.can_be_usize() {
            bail!("Length type isn't numeric!");
        }

        Ok(H2Type::new(alignment, H2Types::LPString(Self {
            length: length,
            character: character,
            renderer: renderer,
        })))
    }

    pub fn new(length: IntegerReader, character: CharacterReader, renderer: CharacterRenderer) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, length, character, renderer)
    }

    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<Character>)> {
        // TODO: This should be usize
        let length = self.length.read(offset.get_dynamic()?)?.as_usize()?;
        let mut position = offset.position() + self.length.size() as u64;

        let mut result = Vec::new();
        for _ in 0..length {
            let character = self.character.read(offset.at(position).get_dynamic()?)?;

            result.push(character);
            position = position + character.size() as u64;
        }

        Ok((position - offset.position(), result))
    }
}

impl H2TypeTrait for LPString {
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

    use generic_number::{Context, Endian, IntegerReader, CharacterReader, CharacterFormatter, CharacterReplacementPolicy, CharacterUnprintableOption};

    use crate::composite::H2Array;

    #[test]
    fn test_utf8_lpstring() -> SimpleResult<()> {
        //                     --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x00\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));


        let a = LPString::new(
            IntegerReader::U16(Endian::Big),
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_lpstring() -> SimpleResult<()> {
        let data = b"\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = LPString::new(
            IntegerReader::U8,
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;

        // Ensure it can display
        assert_eq!("\"\"", a.to_display(offset)?);

        // Ensure it can resolve (this was breaking due to the string being an
        // empty array)
        a.resolve(offset, None)?;

        Ok(())
    }

    #[test]
    fn test_blank_lpstring() -> SimpleResult<()> {
        let data = b"".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = LPString::new(
            IntegerReader::U8,
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;
        assert!(a.to_display(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //                 --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a: H2Type = LPString::new(
            IntegerReader::U8,
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;

        let resolved = a.resolve(offset, None)?;

        // The second child should be an array of the characters
        assert_eq!("\"AB❄☢𝄞😈÷\"", resolved.display);
        assert_eq!(0..19, resolved.actual_range);

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"\x02hi\x03bye\x04test".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Array::new(3, LPString::new(
          IntegerReader::U8,
          CharacterReader::ASCII,
          CharacterFormatter::pretty_str_character(),
        )?)?;

        assert_eq!(12, t.actual_size(offset)?);
        assert_eq!("[ \"hi\", \"bye\", \"test\" ]", t.to_display(offset)?);

        Ok(())
    }

    #[test]
    fn test_character_renderer() -> SimpleResult<()> {
        let data = b"\x03\x41\x10\x09".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = LPString::new(IntegerReader::U8, CharacterReader::ASCII, CharacterFormatter::pretty_str_character())?;
        assert_eq!("\"A\\x10\\t\"", a.to_display(offset)?);

        let a = LPString::new(IntegerReader::U8, CharacterReader::ASCII, CharacterFormatter::new_character(
                false, // show_single_quotes
                CharacterReplacementPolicy::ReplaceEverything,
                CharacterUnprintableOption::URLEncode,

        ))?;
        assert_eq!("\"%41%10%09\"", a.to_display(offset)?);

        Ok(())
    }
}
