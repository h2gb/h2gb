use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};
use std::mem;

use crate::{Context, Endian, Integer};

/// Defines how data is read from a [`Context`] to produce an [`Integer`].
///
/// Importantly, this can be serialized, which means it can be stored and
/// re-used in the future.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntegerReader {
    /// Unsigned 8-bit integer
    U8,

    /// Unsigned 16-bit integer
    U16(Endian),

    /// Unsigned 24-bit integer
    U24(Endian),

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
}

impl IntegerReader {
    /// Read the chosen value at the given [`Context`].
    ///
    /// The `Context` has the offset embedded, and the [`IntegerReader`] has the
    /// [`Endian`] built-in.
    ///
    /// If successful, this creates a [`Integer`], whose datatype will
    /// match the type that we chose in this struct.
    pub fn read(self, context: Context) -> SimpleResult<Integer> {
        match self {
            Self::I8             => Ok(Integer::from(context.read_i8()?)),
            Self::I16(endian)    => Ok(Integer::from(context.read_i16(endian)?)),
            Self::I32(endian)    => Ok(Integer::from(context.read_i32(endian)?)),
            Self::I64(endian)    => Ok(Integer::from(context.read_i64(endian)?)),
            Self::I128(endian)   => Ok(Integer::from(context.read_i128(endian)?)),

            Self::U8             => Ok(Integer::from(context.read_u8()?)),
            Self::U16(endian)    => Ok(Integer::from(context.read_u16(endian)?)),
            Self::U24(endian)    => Ok(Integer::from(context.read_u24(endian)?)),
            Self::U32(endian)    => Ok(Integer::from(context.read_u32(endian)?)),
            Self::U64(endian)    => Ok(Integer::from(context.read_u64(endian)?)),
            Self::U128(endian)   => Ok(Integer::from(context.read_u128(endian)?)),
        }
    }

    /// The size - in bytes - that will be read by [`Self::read`].
    pub fn size(self) -> usize {
        match self {
            Self::U8      => mem::size_of::<u8>(),
            Self::U16(_)  => mem::size_of::<u16>(),
            Self::U24(_)  => mem::size_of::<u8>() + mem::size_of::<u16>(),
            Self::U32(_)  => mem::size_of::<u32>(),
            Self::U64(_)  => mem::size_of::<u64>(),
            Self::U128(_) => mem::size_of::<u128>(),

            Self::I8      => mem::size_of::<i8>(),
            Self::I16(_)  => mem::size_of::<i16>(),
            Self::I32(_)  => mem::size_of::<i32>(),
            Self::I64(_)  => mem::size_of::<i64>(),
            Self::I128(_) => mem::size_of::<i128>(),
        }
    }

    /// Is the type compatible with [`usize`]?
    ///
    /// Dynamically determine this based on [`mem::size_of`]
    pub fn can_be_usize(self) -> bool {
        match self {
            Self::U8         => (self.size() <= mem::size_of::<usize>()),
            Self::U16(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U24(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U32(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U64(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U128(_)    => (self.size() <= mem::size_of::<usize>()),

            Self::I8         => false,
            Self::I16(_)     => false,
            Self::I32(_)     => false,
            Self::I64(_)     => false,
            Self::I128(_)    => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_u32() -> SimpleResult<()> {
        let data = b"\x12\x34\x56\x78\x9a\xbc\xde\xf0".to_vec();

        assert_eq!(Integer::from(0x12345678u32), IntegerReader::U32(Endian::Big).read(Context::new_at(&data, 0))?);
        assert_eq!(Integer::from(0x9abcdef0u32), IntegerReader::U32(Endian::Big).read(Context::new_at(&data, 4))?);

        assert_eq!(Integer::from(0x78563412u32), IntegerReader::U32(Endian::Little).read(Context::new_at(&data, 0))?);
        assert_eq!(Integer::from(0xf0debc9au32), IntegerReader::U32(Endian::Little).read(Context::new_at(&data, 4))?);

        Ok(())
    }

    #[test]
    fn test_buffer_too_short() -> SimpleResult<()> {
        let data = b"".to_vec();
        assert!(IntegerReader::I8.read(Context::new(&data)).is_err());

        let data = b"A".to_vec();
        assert!(IntegerReader::I8.read(Context::new(&data)).is_ok());
        assert!(IntegerReader::I16(Endian::Big).read(Context::new(&data)).is_err());

        let data = b"AAA".to_vec();
        assert!(IntegerReader::I16(Endian::Big).read(Context::new(&data)).is_ok());
        assert!(IntegerReader::I32(Endian::Big).read(Context::new(&data)).is_err());

        let data = b"AAAAAAA".to_vec();
        assert!(IntegerReader::I32(Endian::Big).read(Context::new(&data)).is_ok());
        assert!(IntegerReader::I64(Endian::Big).read(Context::new(&data)).is_err());

        let data = b"AAAAAAAAAAAAAAA".to_vec();
        assert!(IntegerReader::I64(Endian::Big).read(Context::new(&data)).is_ok());
        assert!(IntegerReader::I128(Endian::Big).read(Context::new(&data)).is_err());

        Ok(())
    }
}
