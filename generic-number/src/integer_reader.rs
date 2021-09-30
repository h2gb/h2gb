use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};
use std::mem;

use crate::{Context, Endian, Integer};

/// Defines how data is read from a [`Context`] to produce a [`GenericNumber`].
///
/// Importantly, this can be serialized, which means it can be stored and
/// re-used in the future.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntegerReader {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

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
