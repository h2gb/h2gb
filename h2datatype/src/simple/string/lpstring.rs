use std::iter::FromIterator;
use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::{Context, IntegerReader, Character, CharacterReader, CharacterRenderer};

use crate::{H2Type, H2Types, H2TypeTrait, Alignment, Data};

/// Defines a length-prefixed string.
///
/// This is a string with a numerical prefix that denotes the length of the
/// string (in *characters*). The length is any numerical value as defined in
/// [`generic_number::IntegerReader`] that `can_be_usize()`, and the
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

    fn analyze(&self, context: Context) -> SimpleResult<(usize, Vec<Character>)> {
        let length = self.length.read(context)?.as_usize()?;
        let mut position = context.position() + self.length.size();

        let mut result = Vec::new();
        for _ in 0..length {
            let character = self.character.read(context.at(position))?;

            result.push(character);
            position = position + character.size();
        }

        Ok((position - context.position(), result))
    }
}

impl H2TypeTrait for LPString {
    fn base_size(&self, context: Context, _data: &Data) -> SimpleResult<usize> {
        Ok(self.analyze(context)?.0)
    }

    fn can_be_string(&self) -> bool {
        true
    }

    fn to_string(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        // Get the length so we can truncate
        let (_, chars) = self.analyze(context)?;

        // Convert into a string
        Ok(String::from_iter(chars.into_iter().map(|c| self.renderer.render(c))))
    }

    fn to_display(&self, context: Context, data: &Data) -> SimpleResult<String> {
        Ok(format!("\"{}\"", self.to_string(context, data)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use generic_number::{Context, Endian, IntegerReader, CharacterReader, CharacterFormatter, CharacterReplacementPolicy, CharacterUnprintableOption};

    use crate::composite::H2Array;

    #[test]
    fn test_utf8_lpstring() -> SimpleResult<()> {
        //                     --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x00\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let context = Context::new(&data);


        let a = LPString::new(
            IntegerReader::U16(Endian::Big),
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(context, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_lpstring() -> SimpleResult<()> {
        let data = b"\x00\x41".to_vec();
        let context = Context::new(&data);

        let a = LPString::new(
            IntegerReader::U8,
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;

        // Ensure it can display
        assert_eq!("\"\"", a.to_display(context, &Data::default())?);

        // Ensure it can resolve (this was breaking due to the string being an
        // empty array)
        a.resolve(context, None, &Data::default())?;

        Ok(())
    }

    #[test]
    fn test_blank_lpstring() -> SimpleResult<()> {
        let data = b"".to_vec();
        let context = Context::new(&data);

        let a = LPString::new(
            IntegerReader::U8,
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;
        assert!(a.to_display(context, &Data::default()).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //                 --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let context = Context::new(&data);

        let a: H2Type = LPString::new(
            IntegerReader::U8,
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )?;

        let resolved = a.resolve(context, None, &Data::default())?;

        // The second child should be an array of the characters
        assert_eq!("\"AB❄☢𝄞😈÷\"", resolved.display);
        assert_eq!(0..19, resolved.actual_range);

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"\x02hi\x03bye\x04test".to_vec();
        let context = Context::new(&data);

        let t = H2Array::new(3, LPString::new(
          IntegerReader::U8,
          CharacterReader::ASCII,
          CharacterFormatter::pretty_str_character(),
        )?)?;

        assert_eq!(12, t.base_size(context, &Data::default())?);
        assert_eq!("[ \"hi\", \"bye\", \"test\" ]", t.to_display(context, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_character_renderer() -> SimpleResult<()> {
        let data = b"\x03\x41\x10\x09".to_vec();
        let context = Context::new(&data);

        let a = LPString::new(IntegerReader::U8, CharacterReader::ASCII, CharacterFormatter::pretty_str_character())?;
        assert_eq!("\"A\\x10\\t\"", a.to_display(context, &Data::default())?);

        let a = LPString::new(IntegerReader::U8, CharacterReader::ASCII, CharacterFormatter::new_character(
                false, // show_single_quotes
                CharacterReplacementPolicy::ReplaceEverything,
                CharacterUnprintableOption::URLEncode,

        ))?;
        assert_eq!("\"%41%10%09\"", a.to_display(context, &Data::default())?);

        Ok(())
    }
}
