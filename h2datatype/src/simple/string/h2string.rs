use std::iter::FromIterator;
use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::{Context, Character, CharacterReader, CharacterRenderer};

use crate::{H2Type, H2TypeTrait, Alignment, Data};

/// Defines a string with a configured length.
///
/// The length (in characters) is chosen when creating the type. The length in
/// bytes may be longer if the character type is non-ASCII, however. See
/// [`generic_number::CharacterReader`] for a list of possible character
/// types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2String {
    length: usize,

    character: CharacterReader,

    #[serde(default)]
    renderer: CharacterRenderer,

    #[serde(skip_serializing_if = "Option::is_none")]
    alignment: Option<Alignment>,
}

impl From<H2String> for H2Type {
    fn from(t: H2String) -> H2Type {
        H2Type::H2String(t)
    }
}

impl H2String {
    pub fn new_aligned(alignment: Option<Alignment>, length_in_characters: usize, character: impl Into<CharacterReader>, renderer: impl Into<CharacterRenderer>) -> SimpleResult<Self> {
        if length_in_characters == 0 {
            bail!("Length must be at least 1 character long");
        }

        Ok(Self {
            length: length_in_characters,
            character: character.into(),
            renderer: renderer.into(),
            alignment: alignment,
        })
    }

    pub fn new(length_in_characters: usize, character: impl Into<CharacterReader>, renderer: impl Into<CharacterRenderer>) -> SimpleResult<Self> {
        Self::new_aligned(None, length_in_characters, character, renderer)
    }


    fn analyze(&self, context: Context) -> SimpleResult<(usize, Vec<Character>)> {
        let mut position = context.position();
        let mut result = Vec::new();

        for _ in 0..self.length {
            let this_context = context.at(position);

            let this_character = self.character.read(this_context)?;

            result.push(this_character);
            position = position + this_character.size();
        }

        Ok((position - context.position(), result))
    }
}

impl H2TypeTrait for H2String {
    fn base_size(&self, context: Context, _data: &Data) -> SimpleResult<usize> {
        match self.character.size() {
            Some(s) => Ok(s * self.length),
            None => Ok(self.analyze(context)?.0),
        }
    }

    fn can_be_string(&self) -> bool {
        true
    }

    fn to_string(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        // Get the length so we can truncate
        let (_, chars) = self.analyze(context)?;

        // Convert into a string
        Ok(String::from_iter(chars.into_iter().map(|c| self.renderer.render_character(c))))
    }

    fn to_display(&self, context: Context, data: &Data) -> SimpleResult<String> {
        Ok(format!("\"{}\"", self.to_string(context, data)?))
    }

    fn alignment(&self) -> Option<Alignment> {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, CharacterFormatter, CharacterReplacementPolicy, CharacterUnprintableOption};
    use pretty_assertions::assert_eq;

    use crate::composite::H2Array;

    #[test]
    fn test_utf8_lstring() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let context = Context::new(&data);

        let a = H2String::new(7, CharacterReader::UTF8, CharacterFormatter::new_pretty_str())?;
        assert_eq!("\"AB❄☢𝄞😈÷\"", a.to_display(context, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_lstring() -> SimpleResult<()> {
        assert!(H2String::new(0, CharacterReader::UTF8, CharacterFormatter::new_pretty_str()).is_err());

        Ok(())
    }

    #[test]
    fn test_too_long_lstring() -> SimpleResult<()> {
        let data = b"A".to_vec();
        let context = Context::new(&data);

        let a = H2String::new(2, CharacterReader::UTF8, CharacterFormatter::new_pretty_str())?;
        assert!(a.to_display(context, &Data::default()).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let context = Context::new(&data);

        let a = H2String::new(7, CharacterReader::UTF8, CharacterFormatter::new_pretty_str())?;
        let resolved = a.resolve(context, None, &Data::default())?;

        assert_eq!("\"AB❄☢𝄞😈÷\"", resolved.display);

        Ok(())
    }

    #[test]
    fn test_starting_non_zero_offset() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        let t = H2Array::new(4, H2String::new(4, CharacterReader::ASCII, CharacterFormatter::new_pretty_str())?)?;

        assert_eq!(16, t.base_size(context, &Data::default()).unwrap());
        assert_eq!("[ \"AAAA\", \"BBBB\", \"CCCC\", \"DDDD\" ]", t.to_display(context, &Data::default()).unwrap());

        Ok(())
    }

    #[test]
    fn test_character_renderer() -> SimpleResult<()> {
        let data = b"\x41\x10\x09".to_vec();
        let context = Context::new(&data);

        let a = H2String::new(3, CharacterReader::ASCII, CharacterFormatter::new_pretty_str())?;
        assert_eq!("\"A\\x10\\t\"", a.to_display(context, &Data::default())?);

        let a = H2String::new(3, CharacterReader::ASCII, CharacterFormatter::new(
                false, // show_single_quotes
                CharacterReplacementPolicy::ReplaceEverything,
                CharacterUnprintableOption::URLEncode,

        ))?;
        assert_eq!("\"%41%10%09\"", a.to_display(context, &Data::default())?);

        Ok(())
    }
}
