use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::{Context, Endian, Character};

/// Defines how data is read from a [`Context`] to produce a [`Character`].
///
/// Importantly, this can be serialized, which means it can be stored and
/// re-used in the future.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CharacterReader {
    /// 8-bit ASCII Character
    ASCII,

    /// 8 - 32-bit UTF8 character
    UTF8,

    /// 16 or 32-bit UTF16 character
    UTF16(Endian),

    /// 32-bit UTF32 character
    UTF32(Endian),
}

impl CharacterReader {
    /// Read the chosen value at the given [`Context`].
    ///
    /// The `Context` has the offset embedded, and the [`CharacterReader`] has the
    /// [`Endian`] built-in.
    ///
    /// If successful, this creates a [`Character`] that represents the
    /// character of the requested type read from the `Context`.
    /// match the type that we chose in this struct.
    pub fn read(self, context: Context) -> SimpleResult<Character> {
        match self {
            // We confidently know that ASCII is 1 byte
            Self::ASCII          => Ok(Character::from((context.read_ascii()?, 1))),

            // We need to read UTF8 to know its size
            Self::UTF8          => {
                let (n, c) = context.read_utf8()?;
                Ok(Character::from((c, n)))
            }

            // We also need to read UTF16 to know its size
            Self::UTF16(endian) => {
                let (n, c) = context.read_utf16(endian)?;
                Ok(Character::from((c, n)))
            }

            // UTF-32 does not change size, but it does have an endian
            Self::UTF32(endian)  => Ok(Character::from((context.read_utf32(endian)?, 4))),
        }
    }

    /// The size - in bytes - that will be read by [`Self::read`].
    ///
    /// Note that not all types have a pre-defined size; those return [`None`].
    pub fn size(self) -> Option<usize> {
        match self {
            Self::ASCII    => Some(1),
            Self::UTF8     => None,
            Self::UTF16(_) => None,
            Self::UTF32(_) => Some(4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use crate::CharacterReader;

    #[test]
    fn test_ascii() -> SimpleResult<()> {
        let data = b"\x41\x42\xff".to_vec();

        let c = CharacterReader::ASCII.read(Context::new_at(&data, 0))?;
        assert_eq!(c.size(), 1);
        assert_eq!(c.as_char(), 'A');

        let c = CharacterReader::ASCII.read(Context::new_at(&data, 1))?;
        assert_eq!(c.size(), 1);
        assert_eq!(c.as_char(), 'B');

        let c = CharacterReader::ASCII.read(Context::new_at(&data, 2))?;
        assert_eq!(c.size(), 1);
        assert_eq!(c.as_char(), 0xff as char);

        Ok(())
    }

    #[test]
    fn test_utf8() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        //             A   B   ❄           ☢           𝄞               😈              ÷
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();

        let c = CharacterReader::UTF8.read(Context::new_at(&data, 0))?;
        assert_eq!(c.size(), 1);
        assert_eq!(c.as_char(), 'A');

        let c = CharacterReader::UTF8.read(Context::new_at(&data, 1))?;
        assert_eq!(c.size(), 1);
        assert_eq!(c.as_char(), 'B');

        let c = CharacterReader::UTF8.read(Context::new_at(&data, 2))?;
        assert_eq!(c.size(), 3);
        assert_eq!(c.as_char(), '❄');

        Ok(())
    }

    #[test]
    fn test_utf16_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let c = Context::new(&data);

        // Single
        assert_eq!(2,   CharacterReader::UTF16(Endian::Big).read(c.at(0))?.size());
        assert_eq!('A', CharacterReader::UTF16(Endian::Big).read(c.at(0))?.as_char());

        assert_eq!(2,   CharacterReader::UTF16(Endian::Big).read(c.at(2))?.size());
        assert_eq!('B', CharacterReader::UTF16(Endian::Big).read(c.at(2))?.as_char());

        assert_eq!(2,   CharacterReader::UTF16(Endian::Big).read(c.at(4))?.size());
        assert_eq!('❄', CharacterReader::UTF16(Endian::Big).read(c.at(4))?.as_char());

        assert_eq!(2,   CharacterReader::UTF16(Endian::Big).read(c.at(6))?.size());
        assert_eq!('☢', CharacterReader::UTF16(Endian::Big).read(c.at(6))?.as_char());

        // Double
        assert_eq!(4,   CharacterReader::UTF16(Endian::Big).read(c.at(8))?.size());
        assert_eq!('𝄞', CharacterReader::UTF16(Endian::Big).read(c.at(8))?.as_char());

        assert_eq!(4,   CharacterReader::UTF16(Endian::Big).read(c.at(12))?.size());
        assert_eq!('😈', CharacterReader::UTF16(Endian::Big).read(c.at(12))?.as_char());

        Ok(())
    }

    #[test]
    fn test_utf16_little_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x41\x00\x42\x00\x44\x27\x22\x26\x34\xd8\x1e\xdd\x3d\xd8\x08\xde".to_vec();
        let c = Context::new(&data);

        // Single
        assert_eq!(2,   CharacterReader::UTF16(Endian::Little).read(c.at(0))?.size());
        assert_eq!('A', CharacterReader::UTF16(Endian::Little).read(c.at(0))?.as_char());

        assert_eq!(2,   CharacterReader::UTF16(Endian::Little).read(c.at(2))?.size());
        assert_eq!('B', CharacterReader::UTF16(Endian::Little).read(c.at(2))?.as_char());

        assert_eq!(2,   CharacterReader::UTF16(Endian::Little).read(c.at(4))?.size());
        assert_eq!('❄', CharacterReader::UTF16(Endian::Little).read(c.at(4))?.as_char());

        assert_eq!(2,   CharacterReader::UTF16(Endian::Little).read(c.at(6))?.size());
        assert_eq!('☢', CharacterReader::UTF16(Endian::Little).read(c.at(6))?.as_char());

        // Double
        assert_eq!(4,   CharacterReader::UTF16(Endian::Little).read(c.at(8))?.size());
        assert_eq!('𝄞', CharacterReader::UTF16(Endian::Little).read(c.at(8))?.as_char());

        assert_eq!(4,    CharacterReader::UTF16(Endian::Little).read(c.at(12))?.size());
        assert_eq!('😈', CharacterReader::UTF16(Endian::Little).read(c.at(12))?.as_char());

        Ok(())
    }

    #[test]
    fn test_utf32_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x41\x00\x00\x00\x42\x00\x00\x27\x44\x00\x00\x26\x22\x00\x01\xD1\x1E\x00\x01\xF6\x08".to_vec();
        let c = Context::new(&data);

        assert_eq!(4,   CharacterReader::UTF32(Endian::Big).read(c.at(0))?.size());
        assert_eq!('A', CharacterReader::UTF32(Endian::Big).read(c.at(0))?.as_char());

        assert_eq!(4,   CharacterReader::UTF32(Endian::Big).read(c.at(4))?.size());
        assert_eq!('B', CharacterReader::UTF32(Endian::Big).read(c.at(4))?.as_char());

        assert_eq!(4,   CharacterReader::UTF32(Endian::Big).read(c.at(8))?.size());
        assert_eq!('❄', CharacterReader::UTF32(Endian::Big).read(c.at(8))?.as_char());

        assert_eq!(4,   CharacterReader::UTF32(Endian::Big).read(c.at(12))?.size());
        assert_eq!('☢', CharacterReader::UTF32(Endian::Big).read(c.at(12))?.as_char());

        assert_eq!(4,   CharacterReader::UTF32(Endian::Big).read(c.at(16))?.size());
        assert_eq!('𝄞', CharacterReader::UTF32(Endian::Big).read(c.at(16))?.as_char());

        assert_eq!(4,    CharacterReader::UTF32(Endian::Big).read(c.at(20))?.size());
        assert_eq!('😈', CharacterReader::UTF32(Endian::Big).read(c.at(20))?.as_char());

        Ok(())
    }
}
