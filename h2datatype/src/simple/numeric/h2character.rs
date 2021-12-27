use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use generic_number::{Context, Character, CharacterReader, CharacterRenderer, CharacterFormatter};

use crate::{Alignment, H2Type, H2Types, H2TypeTrait};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Character {
    /// The sign, signedness, and endianness of the value.
    reader: CharacterReader,

    /// How the value is to be displayed.
    ///
    /// This is created by the various --Formatter modules in GenericNumber.
    /// For example, [`DefaultFormatter::new()`] or [`HexFormatter::pretty()`].
    renderer: CharacterRenderer,
}

impl H2Character {
    pub fn new_aligned(alignment: Alignment, reader: CharacterReader, renderer: CharacterRenderer) -> H2Type {
        H2Type::new(alignment, H2Types::H2Character(Self {
            reader: reader,
            renderer: renderer,
        }))
    }

    pub fn new(reader: CharacterReader, renderer: CharacterRenderer) -> H2Type {
        Self::new_aligned(Alignment::None, reader, renderer)
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as ASCII, formats the characters in the style of `'a'`.
    pub fn new_ascii() -> H2Type {
        Self::new(
            CharacterReader::ASCII,
            CharacterFormatter::pretty_character(),
        )
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as ASCII, formats as just a letter, like `a`.
    pub fn new_ascii_string() -> H2Type {
        Self::new(
            CharacterReader::ASCII,
            CharacterFormatter::pretty_str_character(),
        )
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as UTF8, formats the characters in the style of `'a'`.
    pub fn new_utf8() -> H2Type {
        Self::new(
            CharacterReader::UTF8,
            CharacterFormatter::pretty_character(),
        )
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as UTF8, formats the characters in the style of `a`.
    pub fn new_utf8_string() -> H2Type {
        Self::new(
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str_character(),
        )
    }
}

impl H2TypeTrait for H2Character {
    fn base_size(&self, context: Context) -> SimpleResult<usize> {
        match self.reader.size() {
            Some(v) => Ok(v),
            None    => Ok(self.reader.read(context)?.size()),
        }
    }

    fn to_display(&self, context: Context) -> SimpleResult<String> {
        Ok(self.renderer.render(self.to_character(context)?))
    }

    fn can_be_character(&self) -> bool {
        true
    }

    fn to_character(&self, context: Context) -> SimpleResult<Character> {
        self.reader.read(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_ascii() -> SimpleResult<()> {
        let data = b"\x41\x42\xff".to_vec();

        assert_eq!("'A'",     H2Character::new_ascii().to_display(Context::new_at(&data, 0))?);
        assert_eq!("'B'",     H2Character::new_ascii().to_display(Context::new_at(&data, 1))?);
        assert_eq!(format!("'{}'", 0xFF as char), H2Character::new_ascii().to_display(Context::new_at(&data, 2))?);

        Ok(())
    }

    #[test]
    fn test_utf8() -> SimpleResult<()> {
        //             --  --  ----------
        //             A   B   ❄
        let data = b"\x41\x42\xE2\x9D\x84".to_vec();

        assert_eq!("'A'", H2Character::new_utf8().to_display(Context::new_at(&data, 0))?);
        assert_eq!(1,     H2Character::new_utf8().base_size(Context::new_at(&data, 0))?);

        assert_eq!("'B'", H2Character::new_utf8().to_display(Context::new_at(&data, 1))?);
        assert_eq!(1,     H2Character::new_utf8().base_size(Context::new_at(&data, 1))?);

        assert_eq!("'❄'", H2Character::new_utf8().to_display(Context::new_at(&data, 2))?);
        assert_eq!(3,     H2Character::new_utf8().base_size(Context::new_at(&data, 2))?);

        Ok(())
    }
}
