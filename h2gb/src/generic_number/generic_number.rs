use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

/// A number that can be any of the primitive types.
///
/// The goal of creating this enum is to wrap around *any* generic type, with
/// serialize, deserialize, and transparent conversion to [`u64`] and [`i64`].
///
/// Typically, you'd use a [`crate::generic_number::GenericReader`] to create a
/// [`GenericNumber`], then a [`crate::generic_number::GenericFormatter`] to
/// render it. All three of those classes can be serialized, so this operation
/// is always repeatable!
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GenericNumber {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    F32(f32),
    F64(f64),

    // Explicitly store the size of the character, because the source (whether
    // it was UTF8, UTF16, ASCII, etc.) is lost
    Char(char, usize),
}

// Simplify converting from various basic types - generally, these shouldn't be
// used directly except for testing!
impl From<u8>   for GenericNumber { fn from(o: u8)   -> Self { Self::U8(o)   } }
impl From<u16>  for GenericNumber { fn from(o: u16)  -> Self { Self::U16(o)  } }
impl From<u32>  for GenericNumber { fn from(o: u32)  -> Self { Self::U32(o)  } }
impl From<u64>  for GenericNumber { fn from(o: u64)  -> Self { Self::U64(o)  } }
impl From<u128> for GenericNumber { fn from(o: u128) -> Self { Self::U128(o) } }
impl From<i8>   for GenericNumber { fn from(o: i8)   -> Self { Self::I8(o)   } }
impl From<i16>  for GenericNumber { fn from(o: i16)  -> Self { Self::I16(o)  } }
impl From<i32>  for GenericNumber { fn from(o: i32)  -> Self { Self::I32(o)  } }
impl From<i64>  for GenericNumber { fn from(o: i64)  -> Self { Self::I64(o)  } }
impl From<i128> for GenericNumber { fn from(o: i128) -> Self { Self::I128(o) } }
impl From<f32>  for GenericNumber { fn from(o: f32)  -> Self { Self::F32(o)  } }
impl From<f64>  for GenericNumber { fn from(o: f64)  -> Self { Self::F64(o)  } }

impl From<(char, usize)> for GenericNumber { fn from(o: (char, usize)) -> Self { Self::Char(o.0, o.1) } }

impl GenericNumber {
    /// The size - in bytes - of the type.
    pub fn size(self) -> usize {
        match self {
            Self::U8(_)   => 1,
            Self::U16(_)  => 2,
            Self::U32(_)  => 4,
            Self::U64(_)  => 8,
            Self::U128(_) => 16,

            Self::I8(_)   => 1,
            Self::I16(_)  => 2,
            Self::I32(_)  => 4,
            Self::I64(_)  => 8,
            Self::I128(_) => 16,

            Self::F32(_)  => 4,
            Self::F64(_)  => 8,

            // Characters know their own size
            Self::Char(_, n) => n,
        }
    }

    /// Is the type compatible with [`u64`]?
    ///
    /// That is, unsigned and no larger than 64 bits.
    pub fn can_be_u64(self) -> bool {
        match self {
            Self::U8(_)      => true,
            Self::U16(_)     => true,
            Self::U32(_)     => true,
            Self::U64(_)     => true,
            Self::U128(_)    => false,

            Self::I8(_)      => false,
            Self::I16(_)     => false,
            Self::I32(_)     => false,
            Self::I64(_)     => false,
            Self::I128(_)    => false,

            Self::F32(_)     => false,
            Self::F64(_)     => false,

            Self::Char(_, _) => true,
        }
    }

    /// Attempt to convert to a [`u64`].
    pub fn as_u64(self) -> SimpleResult<u64> {
        match self {
            Self::U8(v)        => Ok(v as u64),
            Self::U16(v)       => Ok(v as u64),
            Self::U32(v)       => Ok(v as u64),
            Self::U64(v)       => Ok(v as u64),

            // None of these can become u32
            Self::U128(_) => bail!("Can't convert u128 into u64"),
            Self::I8(_)   => bail!("Can't convert i8 (signed) into u64"),
            Self::I16(_)  => bail!("Can't convert i16 (signed) into u64"),
            Self::I32(_)  => bail!("Can't convert i32 (signed) into u64"),
            Self::I64(_)  => bail!("Can't convert i64 (signed) into u64"),
            Self::I128(_) => bail!("Can't convert i128 (signed) into u64"),
            Self::F32(_)  => bail!("Can't convert floating point into u64"),
            Self::F64(_)  => bail!("Can't convert floating point into u64"),

            // Let a character be a u64, I don't see why not?
            Self::Char(v, _)   => Ok(v as u64),
        }
    }

    /// Is the type compatible with [`i64`]?
    ///
    /// That is, signed and no larger than 64 bits.
    pub fn can_be_i64(self) -> bool {
        match self {
            Self::U8(_)      => false,
            Self::U16(_)     => false,
            Self::U32(_)     => false,
            Self::U64(_)     => false,
            Self::U128(_)    => false,

            Self::I8(_)      => true,
            Self::I16(_)     => true,
            Self::I32(_)     => true,
            Self::I64(_)     => true,
            Self::I128(_)    => false,

            Self::F32(_)     => false,
            Self::F64(_)     => false,

            Self::Char(_, _) => false,
        }
    }

    /// Attempt to convert to a [`i64`].
    pub fn as_i64(self) -> SimpleResult<i64> {
        match self {
            Self::I8(v)        => Ok(v as i64),
            Self::I16(v)       => Ok(v as i64),
            Self::I32(v)       => Ok(v as i64),
            Self::I64(v)       => Ok(v as i64),

            // None of these can become i64
            Self::I128(_)    => bail!("Can't convert i128 into i64"),

            Self::U8(_)      => bail!("Can't convert u8 (unsigned) into i64"),
            Self::U16(_)     => bail!("Can't convert u16 (unsigned) into i64"),
            Self::U32(_)     => bail!("Can't convert u32 (unsigned) into i64"),
            Self::U64(_)     => bail!("Can't convert u64 (unsigned) into i64"),
            Self::U128(_)    => bail!("Can't convert u128 into i64"),
            Self::F32(_)     => bail!("Can't convert floating point into i64"),
            Self::F64(_)     => bail!("Can't convert floating point into i64"),

            Self::Char(_, _) => bail!("Can't convert character into i64"),
        }
    }

    /// Is the type compatible with [`char`]?
    pub fn can_be_char(self) -> bool {
        match self {
            Self::U8(_)      => false,
            Self::U16(_)     => false,
            Self::U32(_)     => false,
            Self::U64(_)     => false,
            Self::U128(_)    => false,

            Self::I8(_)      => false,
            Self::I16(_)     => false,
            Self::I32(_)     => false,
            Self::I64(_)     => false,
            Self::I128(_)    => false,

            Self::F32(_)     => false,
            Self::F64(_)     => false,

            Self::Char(_, _) => true,
        }
    }

    pub fn as_char(self) -> SimpleResult<char> {
        match self {
            Self::I8(_)      => bail!("Can't convert i8 into char"),
            Self::I16(_)     => bail!("Can't convert i16 into char"),
            Self::I32(_)     => bail!("Can't convert i32 into char"),
            Self::I64(_)     => bail!("Can't convert i64 into char"),
            Self::I128(_)    => bail!("Can't convert i128 into char"),
            Self::U8(_)      => bail!("Can't convert u8 (unsigned) into char"),
            Self::U16(_)     => bail!("Can't convert u16 (unsigned) into char"),
            Self::U32(_)     => bail!("Can't convert u32 (unsigned) into char"),
            Self::U64(_)     => bail!("Can't convert u64 (unsigned) into char"),
            Self::U128(_)    => bail!("Can't convert u128 into char"),
            Self::F32(_)     => bail!("Can't convert floating point into char"),
            Self::F64(_)     => bail!("Can't convert floating point into char"),

            Self::Char(c, _) => Ok(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::generic_number::{Context, GenericReader, Endian};

    #[test]
    fn test_to_u64() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

        assert_eq!(0u64,   GenericReader::U8.read(Context::new_at(&data, 0))?.as_u64()?);
        assert_eq!(127u64, GenericReader::U8.read(Context::new_at(&data, 1))?.as_u64()?);
        assert_eq!(128u64, GenericReader::U8.read(Context::new_at(&data, 2))?.as_u64()?);
        assert_eq!(255u64, GenericReader::U8.read(Context::new_at(&data, 3))?.as_u64()?);

        assert_eq!(127u64,               GenericReader::U16(Endian::Big).read(Context::new_at(&data, 0))?.as_u64()?);
        assert_eq!(8356095u64,           GenericReader::U32(Endian::Big).read(Context::new_at(&data, 0))?.as_u64()?);
        assert_eq!(35889154747335171u64, GenericReader::U64(Endian::Big).read(Context::new_at(&data, 0))?.as_u64()?);

        assert!(GenericReader::I8.read(Context::new_at(&data, 0))?.as_u64().is_err());
        assert!(GenericReader::I16(Endian::Big ).read(Context::new_at(&data, 0))?.as_u64().is_err());
        assert!(GenericReader::I32(Endian::Big ).read(Context::new_at(&data, 0))?.as_u64().is_err());
        assert!(GenericReader::I64(Endian::Big ).read(Context::new_at(&data, 0))?.as_u64().is_err());
        assert!(GenericReader::F32(Endian::Big ).read(Context::new_at(&data, 0))?.as_u64().is_err());
        assert!(GenericReader::F64(Endian::Big ).read(Context::new_at(&data, 0))?.as_u64().is_err());
        assert!(GenericReader::U128(Endian::Big).read(Context::new_at(&data, 0))?.as_u64().is_err());

        Ok(())
    }

    #[test]
    fn test_to_i64() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

        assert_eq!(0i64,                    GenericReader::I8.read(Context::new_at(&data, 0))?.as_i64()?);
        assert_eq!(127i64,                  GenericReader::I8.read(Context::new_at(&data, 1))?.as_i64()?);
        assert_eq!(-128i64,                 GenericReader::I8.read(Context::new_at(&data, 2))?.as_i64()?);
        assert_eq!(-1i64,                   GenericReader::I8.read(Context::new_at(&data, 3))?.as_i64()?);

        assert_eq!(127i64,                  GenericReader::I16(Endian::Big).read(Context::new_at(&data, 0))?.as_i64()?);
        assert_eq!(-32768i64,               GenericReader::I16(Endian::Big).read(Context::new_at(&data, 8))?.as_i64()?);

        assert_eq!(8356095i64,              GenericReader::I32(Endian::Big).read(Context::new_at(&data, 0))?.as_i64()?);
        assert_eq!(-2147483648i64,          GenericReader::I32(Endian::Big).read(Context::new_at(&data, 8))?.as_i64()?);

        assert_eq!(35889154747335171i64,    GenericReader::I64(Endian::Big).read(Context::new_at(&data, 0))?.as_i64()?);
        assert_eq!(-9223372036854775808i64, GenericReader::I64(Endian::Big).read(Context::new_at(&data, 8))?.as_i64()?);

        assert!(GenericReader::U8.read(Context::new_at(&data, 0))?.as_i64().is_err());
        assert!(GenericReader::U16(Endian::Big).read(Context::new_at(&data, 0))?.as_i64().is_err());
        assert!(GenericReader::U32(Endian::Big).read(Context::new_at(&data, 0))?.as_i64().is_err());
        assert!(GenericReader::U64(Endian::Big).read(Context::new_at(&data, 0))?.as_i64().is_err());
        assert!(GenericReader::F32(Endian::Big).read(Context::new_at(&data, 0))?.as_i64().is_err());
        assert!(GenericReader::F64(Endian::Big).read(Context::new_at(&data, 0))?.as_i64().is_err());
        assert!(GenericReader::I128(Endian::Big).read(Context::new_at(&data, 0))?.as_i64().is_err());

        Ok(())
    }
}
