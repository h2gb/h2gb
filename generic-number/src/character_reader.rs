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
