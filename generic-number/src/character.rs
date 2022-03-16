use std::fmt;
use serde::{Serialize, Deserialize};

/// Represents a character, maintaining the original size.
///
/// This can represent any type of character - ASCII, UTF-8, etc., with
/// serialize and deserialize support.
///
/// To use this, create a [`crate::CharacterReader`] to read a
/// [`crate::Context`]. That'll produce a `Character`. Then you can render it
/// using a [`crate::CharacterFormatter`] or [`crate::DefaultFormatter`].
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a buffer
/// let buffer = b"\xE2\x9D\x84".to_vec();
///
/// // Create a context that points to the start of the buffer
/// let context = Context::new_at(&buffer, 0);
///
/// // Create a reader that knows how to read a UTF8 character - this reader can
/// // be serialized and used later!
/// let reader = CharacterReader::UTF8;
///
/// // Read from the context into a generic number - this number can be
/// // serialized and used later!
/// let c = reader.read(context).unwrap();
///
/// // Display it using different formatters (these use the pretty defaults) -
/// // these formatters can also be serialized!
/// assert_eq!("'❄'", CharacterFormatter::new_pretty().render_character(c));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Character {
    character: char,

    /// Explicitly store the size of the character, because the source (whether
    /// it was UTF8, UTF16, ASCII, etc.) is lost
    size: usize,
}

/// Implement a simple From
impl From<(char, usize)> for Character { fn from(o: (char, usize)) -> Self { Self { character: o.0, size: o.1 } } }

impl Character {
    /// The size - in bytes - of the type.
    pub fn size(self) -> usize {
        self.size
    }

    /// The character.
    pub fn as_char(self) -> char {
        self.character
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.character, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_character() -> SimpleResult<()> {
        let c = Character::from(('A', 1));
        assert_eq!('A', c.as_char());
        assert_eq!(1, c.size());

        let c = Character::from(('A', 100));
        assert_eq!('A', c.as_char());
        assert_eq!(100, c.size());

        Ok(())
    }
}
