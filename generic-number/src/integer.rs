use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};
use std::{fmt, mem};

/// A number that can be any of the primitive types.
///
/// The goal of creating this enum is to wrap around *any* generic type, with
/// serialize, deserialize, and transparent conversion to [`u64`] and [`i64`].
///
/// Typically, you'd use a [`crate::IntegerReader`] to create a
/// [`Integer`], then a [`crate::GenericFormatter`] to
/// render it. All three of those classes can be serialized, so this operation
/// is always repeatable!
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Integer {
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
}

// Simplify converting from various basic types - generally, these shouldn't be
// used directly except for testing!
impl From<u8>   for Integer { fn from(o: u8)   -> Self { Self::U8(o)   } }
impl From<u16>  for Integer { fn from(o: u16)  -> Self { Self::U16(o)  } }
impl From<u32>  for Integer { fn from(o: u32)  -> Self { Self::U32(o)  } }
impl From<u64>  for Integer { fn from(o: u64)  -> Self { Self::U64(o)  } }
impl From<u128> for Integer { fn from(o: u128) -> Self { Self::U128(o) } }
impl From<i8>   for Integer { fn from(o: i8)   -> Self { Self::I8(o)   } }
impl From<i16>  for Integer { fn from(o: i16)  -> Self { Self::I16(o)  } }
impl From<i32>  for Integer { fn from(o: i32)  -> Self { Self::I32(o)  } }
impl From<i64>  for Integer { fn from(o: i64)  -> Self { Self::I64(o)  } }
impl From<i128> for Integer { fn from(o: i128) -> Self { Self::I128(o) } }

impl Integer {
    /// The size - in bytes - of the type.
    pub fn size(self) -> usize {
        match self {
            Self::U8(_)   => mem::size_of::<u8>(),
            Self::U16(_)  => mem::size_of::<u16>(),
            Self::U32(_)  => mem::size_of::<u32>(),
            Self::U64(_)  => mem::size_of::<u64>(),
            Self::U128(_) => mem::size_of::<u128>(),

            Self::I8(_)   => mem::size_of::<i8>(),
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
            Self::U8(_)      => (self.size() <= mem::size_of::<usize>()),
            Self::U16(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U32(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U64(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U128(_)    => (self.size() <= mem::size_of::<usize>()),

            Self::I8(_)      => false,
            Self::I16(_)     => false,
            Self::I32(_)     => false,
            Self::I64(_)     => false,
            Self::I128(_)    => false,
        }
    }

    /// Attempt to convert to a [`usize`].
    pub fn as_usize(self) -> SimpleResult<usize> {
        if !self.can_be_usize() {
            bail!("Cannot convert {:?} to a usize ({}-bit) value", self, mem::size_of::<usize>() * 8);
        }

        match self {
            Self::U8(v)        => Ok(v as usize),
            Self::U16(v)       => Ok(v as usize),
            Self::U32(v)       => Ok(v as usize),
            Self::U64(v)       => Ok(v as usize),
            Self::U128(v)      => Ok(v as usize),

            // None of these can become u32
            Self::I8(_)   => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I16(_)  => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I32(_)  => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I64(_)  => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I128(_) => bail!("Can't convert {:?} into an unsigned size value", self),
        }
    }

    /// Is the type compatible with [`isize`]?
    ///
    /// That is, signed and no larger than an `isize` value.
    pub fn can_be_isize(self) -> bool {
        match self {
            Self::U8(_)      => false,
            Self::U16(_)     => false,
            Self::U32(_)     => false,
            Self::U64(_)     => false,
            Self::U128(_)    => false,

            Self::I8(_)      => (mem::size_of::<i8>()   <= mem::size_of::<isize>()),
            Self::I16(_)     => (mem::size_of::<i16>()  <= mem::size_of::<isize>()),
            Self::I32(_)     => (mem::size_of::<i32>()  <= mem::size_of::<isize>()),
            Self::I64(_)     => (mem::size_of::<i64>()  <= mem::size_of::<isize>()),
            Self::I128(_)    => (mem::size_of::<i128>() <= mem::size_of::<isize>()),
        }
    }

    /// Attempt to convert to an [`isize`].
    pub fn as_isize(self) -> SimpleResult<isize> {
        if !self.can_be_isize() {
            bail!("Cannot convert {:?} to an isize ({}-bit) value", self, mem::size_of::<isize>() * 8);
        }

        match self {
            Self::U8(_)   => bail!("Can't convert {:?} into a signed size value", self),
            Self::U16(_)  => bail!("Can't convert {:?} into a signed size value", self),
            Self::U32(_)  => bail!("Can't convert {:?} into a signed size value", self),
            Self::U64(_)  => bail!("Can't convert {:?} into a signed size value", self),
            Self::U128(_) => bail!("Can't convert {:?} into a signed size value", self),

            Self::I8(v)        => Ok(v as isize),
            Self::I16(v)       => Ok(v as isize),
            Self::I32(v)       => Ok(v as isize),
            Self::I64(v)       => Ok(v as isize),
            Self::I128(v)      => Ok(v as isize),
        }
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)   => fmt::Display::fmt(&v, f),
            Self::U16(v)  => fmt::Display::fmt(&v, f),
            Self::U32(v)  => fmt::Display::fmt(&v, f),
            Self::U64(v)  => fmt::Display::fmt(&v, f),
            Self::U128(v) => fmt::Display::fmt(&v, f),

            Self::I8(v)   => fmt::Display::fmt(&v, f),
            Self::I16(v)  => fmt::Display::fmt(&v, f),
            Self::I32(v)  => fmt::Display::fmt(&v, f),
            Self::I64(v)  => fmt::Display::fmt(&v, f),
            Self::I128(v) => fmt::Display::fmt(&v, f),
        }
    }
}

impl fmt::LowerHex for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)   => fmt::LowerHex::fmt(&v, f),
            Self::U16(v)  => fmt::LowerHex::fmt(&v, f),
            Self::U32(v)  => fmt::LowerHex::fmt(&v, f),
            Self::U64(v)  => fmt::LowerHex::fmt(&v, f),
            Self::U128(v) => fmt::LowerHex::fmt(&v, f),

            Self::I8(v)   => fmt::LowerHex::fmt(&v, f),
            Self::I16(v)  => fmt::LowerHex::fmt(&v, f),
            Self::I32(v)  => fmt::LowerHex::fmt(&v, f),
            Self::I64(v)  => fmt::LowerHex::fmt(&v, f),
            Self::I128(v) => fmt::LowerHex::fmt(&v, f),
        }
    }
}

impl fmt::UpperHex for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)   => fmt::UpperHex::fmt(&v, f),
            Self::U16(v)  => fmt::UpperHex::fmt(&v, f),
            Self::U32(v)  => fmt::UpperHex::fmt(&v, f),
            Self::U64(v)  => fmt::UpperHex::fmt(&v, f),
            Self::U128(v) => fmt::UpperHex::fmt(&v, f),

            Self::I8(v)   => fmt::UpperHex::fmt(&v, f),
            Self::I16(v)  => fmt::UpperHex::fmt(&v, f),
            Self::I32(v)  => fmt::UpperHex::fmt(&v, f),
            Self::I64(v)  => fmt::UpperHex::fmt(&v, f),
            Self::I128(v) => fmt::UpperHex::fmt(&v, f),
        }
    }
}

impl fmt::Octal for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)   => fmt::Octal::fmt(&v, f),
            Self::U16(v)  => fmt::Octal::fmt(&v, f),
            Self::U32(v)  => fmt::Octal::fmt(&v, f),
            Self::U64(v)  => fmt::Octal::fmt(&v, f),
            Self::U128(v) => fmt::Octal::fmt(&v, f),

            Self::I8(v)   => fmt::Octal::fmt(&v, f),
            Self::I16(v)  => fmt::Octal::fmt(&v, f),
            Self::I32(v)  => fmt::Octal::fmt(&v, f),
            Self::I64(v)  => fmt::Octal::fmt(&v, f),
            Self::I128(v) => fmt::Octal::fmt(&v, f),
        }
    }
}

impl fmt::LowerExp for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)   => fmt::LowerExp::fmt(&v, f),
            Self::U16(v)  => fmt::LowerExp::fmt(&v, f),
            Self::U32(v)  => fmt::LowerExp::fmt(&v, f),
            Self::U64(v)  => fmt::LowerExp::fmt(&v, f),
            Self::U128(v) => fmt::LowerExp::fmt(&v, f),

            Self::I8(v)   => fmt::LowerExp::fmt(&v, f),
            Self::I16(v)  => fmt::LowerExp::fmt(&v, f),
            Self::I32(v)  => fmt::LowerExp::fmt(&v, f),
            Self::I64(v)  => fmt::LowerExp::fmt(&v, f),
            Self::I128(v) => fmt::LowerExp::fmt(&v, f),
        }
    }
}

impl fmt::UpperExp for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)   => fmt::UpperExp::fmt(&v, f),
            Self::U16(v)  => fmt::UpperExp::fmt(&v, f),
            Self::U32(v)  => fmt::UpperExp::fmt(&v, f),
            Self::U64(v)  => fmt::UpperExp::fmt(&v, f),
            Self::U128(v) => fmt::UpperExp::fmt(&v, f),

            Self::I8(v)   => fmt::UpperExp::fmt(&v, f),
            Self::I16(v)  => fmt::UpperExp::fmt(&v, f),
            Self::I32(v)  => fmt::UpperExp::fmt(&v, f),
            Self::I64(v)  => fmt::UpperExp::fmt(&v, f),
            Self::I128(v) => fmt::UpperExp::fmt(&v, f),
        }
    }
}

impl fmt::Binary for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)   => fmt::Binary::fmt(&v, f),
            Self::U16(v)  => fmt::Binary::fmt(&v, f),
            Self::U32(v)  => fmt::Binary::fmt(&v, f),
            Self::U64(v)  => fmt::Binary::fmt(&v, f),
            Self::U128(v) => fmt::Binary::fmt(&v, f),

            Self::I8(v)   => fmt::Binary::fmt(&v, f),
            Self::I16(v)  => fmt::Binary::fmt(&v, f),
            Self::I32(v)  => fmt::Binary::fmt(&v, f),
            Self::I64(v)  => fmt::Binary::fmt(&v, f),
            Self::I128(v) => fmt::Binary::fmt(&v, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::{Context, IntegerReader, Endian, DefaultFormatter};

    #[test]
    fn test_to_usize() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

        assert_eq!(0usize,   IntegerReader::U8.read(Context::new_at(&data, 0))?.as_usize()?);
        assert_eq!(127usize, IntegerReader::U8.read(Context::new_at(&data, 1))?.as_usize()?);
        assert_eq!(128usize, IntegerReader::U8.read(Context::new_at(&data, 2))?.as_usize()?);
        assert_eq!(255usize, IntegerReader::U8.read(Context::new_at(&data, 3))?.as_usize()?);

        assert_eq!(127usize,               IntegerReader::U16(Endian::Big).read(Context::new_at(&data, 0))?.as_usize()?);
        assert_eq!(8356095usize,           IntegerReader::U32(Endian::Big).read(Context::new_at(&data, 0))?.as_usize()?);
        assert_eq!(35889154747335171usize, IntegerReader::U64(Endian::Big).read(Context::new_at(&data, 0))?.as_usize()?);

        assert!(IntegerReader::I8.read(Context::new_at(&data, 0))?.as_usize().is_err());
        assert!(IntegerReader::I16(Endian::Big ).read(Context::new_at(&data, 0))?.as_usize().is_err());
        assert!(IntegerReader::I32(Endian::Big ).read(Context::new_at(&data, 0))?.as_usize().is_err());
        assert!(IntegerReader::I64(Endian::Big ).read(Context::new_at(&data, 0))?.as_usize().is_err());
        assert!(IntegerReader::U128(Endian::Big).read(Context::new_at(&data, 0))?.as_usize().is_err());

        Ok(())
    }

    #[test]
    fn test_to_isize() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

        assert_eq!(0isize,                    IntegerReader::I8.read(Context::new_at(&data, 0))?.as_isize()?);
        assert_eq!(127isize,                  IntegerReader::I8.read(Context::new_at(&data, 1))?.as_isize()?);
        assert_eq!(-128isize,                 IntegerReader::I8.read(Context::new_at(&data, 2))?.as_isize()?);
        assert_eq!(-1isize,                   IntegerReader::I8.read(Context::new_at(&data, 3))?.as_isize()?);

        assert_eq!(127isize,                  IntegerReader::I16(Endian::Big).read(Context::new_at(&data, 0))?.as_isize()?);
        assert_eq!(-32768isize,               IntegerReader::I16(Endian::Big).read(Context::new_at(&data, 8))?.as_isize()?);

        assert_eq!(8356095isize,              IntegerReader::I32(Endian::Big).read(Context::new_at(&data, 0))?.as_isize()?);
        assert_eq!(-2147483648isize,          IntegerReader::I32(Endian::Big).read(Context::new_at(&data, 8))?.as_isize()?);

        assert_eq!(35889154747335171isize,    IntegerReader::I64(Endian::Big).read(Context::new_at(&data, 0))?.as_isize()?);
        assert_eq!(-9223372036854775808isize, IntegerReader::I64(Endian::Big).read(Context::new_at(&data, 8))?.as_isize()?);

        assert!(IntegerReader::U8.read(Context::new_at(&data, 0))?.as_isize().is_err());
        assert!(IntegerReader::U16(Endian::Big).read(Context::new_at(&data, 0))?.as_isize().is_err());
        assert!(IntegerReader::U32(Endian::Big).read(Context::new_at(&data, 0))?.as_isize().is_err());
        assert!(IntegerReader::U64(Endian::Big).read(Context::new_at(&data, 0))?.as_isize().is_err());
        assert!(IntegerReader::I128(Endian::Big).read(Context::new_at(&data, 0))?.as_isize().is_err());

        Ok(())
    }
}
