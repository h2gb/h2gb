use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::generic_number::{Context, Endian, GenericNumber};

/// Defines how data is read from a [`Context`] to produce a [`GenericNumber`].
///
/// Importantly, this can be serialized, which means it can be stored and
/// re-used in the future.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GenericReader {
    /// Unsigned 8-bit integer
    U8,

    /// Unsigned 16-bit integer
    U16(Endian),

    /// Unsigned 32-bit integer
    U32(Endian),

    /// Unsigned 64-bit integer
    U64(Endian),

    /// Unsigned 128-bit integer
    U128(Endian),

    /// Signed 8-bit integer
    I8,

    /// Signed 16-bit integer
    I16(Endian),

    /// Signed 32-bit integer
    I32(Endian),

    /// Signed 64-bit integer
    I64(Endian),

    /// Signed 128-bit integer
    I128(Endian),

    /// Signed 32-bit (aka, single precision) floating point.
    ///
    /// Note: floats can only be displayed as `GenericFormatter::Default` or
    /// `GenericFormatter::Scientific`.
    F32(Endian),

    /// Signed 64-bit (aka, double precision) floating point
    ///
    /// Note: floats can only be displayed as `GenericFormatter::Default` or
    /// `GenericFormatter::Scientific`.
    F64(Endian),

    /// 8-bit ASCII Character
    ASCII,

    /// 8 - 32-bit UTF8 character
    UTF8,

    /// 16 or 32-bit UTF16 character
    UTF16(Endian),

    /// 32-bit UTF32 character
    UTF32(Endian),
}

impl GenericReader {
    /// Read the chosen value at the given [`Context`].
    ///
    /// The `Context` has the offset embedded, and the [`GenericReader`] has the
    /// [`Endian`] built-in.
    ///
    /// If successful, this creates a [`GenericNumber`], whose datatype will
    /// match the type that we chose in this struct.
    pub fn read(self, context: Context) -> SimpleResult<GenericNumber> {
        match self {
            Self::I8             => Ok(GenericNumber::from(context.read_i8()?)),
            Self::I16(endian)    => Ok(GenericNumber::from(context.read_i16(endian)?)),
            Self::I32(endian)    => Ok(GenericNumber::from(context.read_i32(endian)?)),
            Self::I64(endian)    => Ok(GenericNumber::from(context.read_i64(endian)?)),
            Self::I128(endian)   => Ok(GenericNumber::from(context.read_i128(endian)?)),

            Self::U8             => Ok(GenericNumber::from(context.read_u8()?)),
            Self::U16(endian)    => Ok(GenericNumber::from(context.read_u16(endian)?)),
            Self::U32(endian)    => Ok(GenericNumber::from(context.read_u32(endian)?)),
            Self::U64(endian)    => Ok(GenericNumber::from(context.read_u64(endian)?)),
            Self::U128(endian)   => Ok(GenericNumber::from(context.read_u128(endian)?)),

            Self::F32(endian)    => Ok(GenericNumber::from(context.read_f32(endian)?)),
            Self::F64(endian)    => Ok(GenericNumber::from(context.read_f64(endian)?)),

            // We confidently know that ASCII is 1 byte
            Self::ASCII          => Ok(GenericNumber::from((context.read_ascii()?, 1))),

            // We need to read UTF8 to know its size
            Self::UTF8          => {
                let (n, c) = context.read_utf8()?;
                Ok(GenericNumber::from((c, n)))
            }

            // We also need to read UTF16 to know its size
            Self::UTF16(endian) => {
                let (n, c) = context.read_utf16(endian)?;
                Ok(GenericNumber::from((c, n)))
            }

            // UTF-32 does not change size, but it does have an endian
            Self::UTF32(endian)  => Ok(GenericNumber::from((context.read_utf32(endian)?, 4))),
        }
    }

    /// The size - in bytes - that will be read by [`Self::read`].
    ///
    /// Some types can have varying sizes - those return None.
    pub fn size(self) -> Option<usize> {
        match self {
            Self::I8      => Some(1),
            Self::I16(_)  => Some(2),
            Self::I32(_)  => Some(4),
            Self::I64(_)  => Some(8),
            Self::I128(_) => Some(16),

            Self::U8      => Some(1),
            Self::U16(_)  => Some(2),
            Self::U32(_)  => Some(4),
            Self::U64(_)  => Some(8),
            Self::U128(_) => Some(16),

            Self::F32(_)  => Some(4),
            Self::F64(_)  => Some(8),

            Self::ASCII    => Some(1),
            Self::UTF8     => None,
            Self::UTF16(_) => None,
            Self::UTF32(_) => Some(4),
        }
    }

    /// Will the resulting [`GenericNumber`] be compatible with a [`u64`]?
    ///
    /// This is mostly a convenience function, so datatypes we define can
    /// verify that a proper [`GenericReader`] is being used before it actually
    /// reads anything.
    pub fn can_be_u64(self) -> bool {
        match self {
            Self::I8       => false,
            Self::I16(_)   => false,
            Self::I32(_)   => false,
            Self::I64(_)   => false,
            Self::I128(_)  => false,

            Self::U8       => true,
            Self::U16(_)   => true,
            Self::U32(_)   => true,
            Self::U64(_)   => true,
            Self::U128(_)  => false,

            Self::F32(_)   => false,
            Self::F64(_)   => false,

            Self::ASCII    => true,
            Self::UTF8     => true,
            Self::UTF16(_) => true,
            Self::UTF32(_) => true,
        }
    }

    /// Will the resulting [`GenericNumber`] be compatible with a [`i64`]?
    ///
    /// This is mostly a convenience function, so datatypes we define can
    /// verify that a proper [`GenericReader`] is being used before it actually
    /// reads anything.
    pub fn can_be_i64(self) -> bool {
        match self {
            Self::I8       => true,
            Self::I16(_)   => true,
            Self::I32(_)   => true,
            Self::I64(_)   => true,
            Self::I128(_)  => false,

            Self::U8       => false,
            Self::U16(_)   => false,
            Self::U32(_)   => false,
            Self::U64(_)   => false,
            Self::U128(_)  => false,

            Self::F32(_)   => false,
            Self::F64(_)   => false,

            Self::ASCII    => false,
            Self::UTF8     => false,
            Self::UTF16(_) => false,
            Self::UTF32(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    #[test]
    fn test_buffer_too_short() -> SimpleResult<()> {
        let data = b"".to_vec();
        assert!(GenericReader::I8.read(Context::new(&data)).is_err());

        let data = b"A".to_vec();
        assert!(GenericReader::I8.read(Context::new(&data)).is_ok());
        assert!(GenericReader::I16(Endian::Big).read(Context::new(&data)).is_err());

        let data = b"AAA".to_vec();
        assert!(GenericReader::I16(Endian::Big).read(Context::new(&data)).is_ok());
        assert!(GenericReader::I32(Endian::Big).read(Context::new(&data)).is_err());

        let data = b"AAAAAAA".to_vec();
        assert!(GenericReader::I32(Endian::Big).read(Context::new(&data)).is_ok());
        assert!(GenericReader::I64(Endian::Big).read(Context::new(&data)).is_err());

        let data = b"AAAAAAAAAAAAAAA".to_vec();
        assert!(GenericReader::I64(Endian::Big).read(Context::new(&data)).is_ok());
        assert!(GenericReader::I128(Endian::Big).read(Context::new(&data)).is_err());

        Ok(())
    }
}
