use serde::{Serialize, Deserialize};
use simple_error::{SimpleError, SimpleResult, bail};
use std::{fmt, mem};
use std::str::FromStr;
use std::hash::{Hash, Hasher};

/// A number that can be any of the primitive integer types.
///
/// The goal of creating this enum is to wrap around *any* integer type, with
/// serialize, deserialize, and easy conversion / rendering.
///
/// # Creation
///
/// An [`Integer`] can be created in a bunch of different ways depending on
/// your needs.
///
/// The simplest way is using the [`From`] trait - `Integer::From(1u8)` for
/// example. If a function wants an [`Integer`] argument, the best option is to
/// make the argument [`Into<Integer>`], which will transparently work for
/// basically any numeric type.
///
/// The most common way in `h2gb` is by using a [`crate::IntegerReader`], which
/// reads an [`Integer`] from a [`crate::Context`], which represents binary
/// data.
///
/// This also implements [`str::FromStr`], allowing numbers to be read from
/// a string. When possible it'll convert things to a [`usize`] or [`isize`],
/// falling back to larger datatypes as needed. We also support radix prefixes
/// - specifically, `0x` for hex, `0b` for binary, and `0o` for octal. We
/// anticipate using those to store configurations.
///
/// # Usage
///
/// Integers are generally displayed using a formatter such as
/// [`crate::HexFormatter`], which renders a number with a bunch of formatting
/// options.
///
/// More importantly, this implements the whole suite of comparison and ordering
/// traits - `Eq`, `PartialEq`, `Ord`, and so on. Unlike standard Rust, this
/// will endeavour to compare types of different sizes by converting them to the
/// best shared size - as always, [`usize`] and [`isize`] if possible.
///
/// While this is helpful when analyzing binaries in `h2gb`, I probably wouldn't
/// use this with normal programming - there's a reason Rust strongly types
/// stuff!
///
/// # Note on mixed signedness
///
/// When comparing numbers, they are always compared as unsigned u128 values.
/// That means that equations like this technically work:
///
/// ```
/// use generic_number::*;
///
/// assert!(Integer::from(1i8) == Integer::from(1i8));
/// assert!(Integer::from(-1i8) == Integer::from(-1i8));
/// assert!(Integer::from(-1i8) == Integer::from(0xffffffffffffffffffffffffffffffffu128));
/// ```
///
/// We no longer implement ordering, because ordering signed and unsigned
/// numbers together is basically impossible to do sanely.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Integer {
    U8(u8),
    U16(u16),
    U24(u8, u16), // MSB, then LSB
    U32(u32),
    U64(u64),
    U128(u128),
    USize(usize),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    I128(i128),
    ISize(isize),
}

// Simplify converting from various basic types - generally, these shouldn't be
// used directly except for testing!
impl From<u8>    for Integer { fn from(o: u8)   -> Self { Self::U8(o)   } }
impl From<u16>   for Integer { fn from(o: u16)  -> Self { Self::U16(o)  } }
impl From<(u8,u16)> for Integer { fn from(o: (u8, u16)) -> Self { Self::U24(o.0, o.1)  } }
impl From<u32>   for Integer { fn from(o: u32)  -> Self { Self::U32(o)  } }
impl From<u64>   for Integer { fn from(o: u64)  -> Self { Self::U64(o)  } }
impl From<u128>  for Integer { fn from(o: u128) -> Self { Self::U128(o) } }
impl From<usize> for Integer { fn from(o: usize) -> Self { Self::USize(o) } }
impl From<i8>    for Integer { fn from(o: i8)   -> Self { Self::I8(o)   } }
impl From<i16>   for Integer { fn from(o: i16)  -> Self { Self::I16(o)  } }
impl From<i32>   for Integer { fn from(o: i32)  -> Self { Self::I32(o)  } }
impl From<i64>   for Integer { fn from(o: i64)  -> Self { Self::I64(o)  } }
impl From<i128>  for Integer { fn from(o: i128) -> Self { Self::I128(o) } }
impl From<isize> for Integer { fn from(o: isize) -> Self { Self::ISize(o) } }

impl Integer {
    /// The size - in bytes - of the type.
    pub fn size(self) -> usize {
        match self {
            Self::U8(_)    => mem::size_of::<u8>(),
            Self::U16(_)   => mem::size_of::<u16>(),
            Self::U24(_,_) => mem::size_of::<u16>() + mem::size_of::<u8>(),
            Self::U32(_)   => mem::size_of::<u32>(),
            Self::U64(_)   => mem::size_of::<u64>(),
            Self::U128(_)  => mem::size_of::<u128>(),
            Self::USize(_) => mem::size_of::<usize>(),

            Self::I8(_)    => mem::size_of::<i8>(),
            Self::I16(_)   => mem::size_of::<i16>(),
            Self::I32(_)   => mem::size_of::<i32>(),
            Self::I64(_)   => mem::size_of::<i64>(),
            Self::I128(_)  => mem::size_of::<i128>(),
            Self::ISize(_) => mem::size_of::<isize>(),
        }
    }

    /// Is it signed?
    pub fn is_signed(self) -> bool {
        match self {
            Self::U8(_)    => false,
            Self::U16(_)   => false,
            Self::U24(_,_) => false,
            Self::U32(_)   => false,
            Self::U64(_)   => false,
            Self::U128(_)  => false,
            Self::USize(_) => false,

            Self::I8(_)    => true,
            Self::I16(_)   => true,
            Self::I32(_)   => true,
            Self::I64(_)   => true,
            Self::I128(_)  => true,
            Self::ISize(_) => true,
        }
    }

    /// Is the type compatible with [`usize`]?
    ///
    /// Dynamically determine this based on [`mem::size_of`]
    pub fn can_be_usize(self) -> bool {
        match self {
            Self::U8(_)      => (self.size() <= mem::size_of::<usize>()),
            Self::U16(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U24(_,_)   => (self.size() <= mem::size_of::<usize>()),
            Self::U32(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U64(_)     => (self.size() <= mem::size_of::<usize>()),
            Self::U128(_)    => (self.size() <= mem::size_of::<usize>()),
            Self::USize(_)   => true,

            Self::I8(_)      => false,
            Self::I16(_)     => false,
            Self::I32(_)     => false,
            Self::I64(_)     => false,
            Self::I128(_)    => false,
            Self::ISize(_)   => false,
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
            Self::U24(msb,lsb) => Ok(((msb as usize)) << 16 | (lsb as usize)),
            Self::U32(v)       => Ok(v as usize),

            // These may be unreachable - it depends if can_be_usize() passes
            Self::U64(v)       => Ok(v as usize),
            Self::U128(v)      => Ok(v as usize),

            // Easy!
            Self::USize(v)     => Ok(v),

            // None of these can become u32
            Self::I8(_)    => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I16(_)   => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I32(_)   => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I64(_)   => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::I128(_)  => bail!("Can't convert {:?} into an unsigned size value", self),
            Self::ISize(_) => bail!("Can't convert {:?} into an unsigned size value", self),
        }
    }

    /// Is the type compatible with [`isize`]?
    ///
    /// That is, signed and no larger than an `isize` value.
    pub fn can_be_isize(self) -> bool {
        match self {
            Self::U8(_)      => false,
            Self::U16(_)     => false,
            Self::U24(_,_)   => false,
            Self::U32(_)     => false,
            Self::U64(_)     => false,
            Self::U128(_)    => false,
            Self::USize(_)   => false,

            Self::I8(_)      => (mem::size_of::<i8>()   <= mem::size_of::<isize>()),
            Self::I16(_)     => (mem::size_of::<i16>()  <= mem::size_of::<isize>()),
            Self::I32(_)     => (mem::size_of::<i32>()  <= mem::size_of::<isize>()),
            Self::I64(_)     => (mem::size_of::<i64>()  <= mem::size_of::<isize>()),
            Self::I128(_)    => (mem::size_of::<i128>() <= mem::size_of::<isize>()),
            Self::ISize(_)   => true,
        }
    }

    /// Attempt to convert to an [`isize`].
    pub fn as_isize(self) -> SimpleResult<isize> {
        if !self.can_be_isize() {
            bail!("Cannot convert {:?} to an isize ({}-bit) value", self, mem::size_of::<isize>() * 8);
        }

        match self {
            Self::U8(_)    => bail!("Can't convert {:?} into a signed size value", self),
            Self::U16(_)   => bail!("Can't convert {:?} into a signed size value", self),
            Self::U24(_,_) => bail!("Can't convert {:?} into a signed size value", self),
            Self::U32(_)   => bail!("Can't convert {:?} into a signed size value", self),
            Self::U64(_)   => bail!("Can't convert {:?} into a signed size value", self),
            Self::U128(_)  => bail!("Can't convert {:?} into a signed size value", self),
            Self::USize(_) => bail!("Can't convert {:?} into a signed size value", self),

            Self::I8(v)        => Ok(v as isize),
            Self::I16(v)       => Ok(v as isize),
            Self::I32(v)       => Ok(v as isize),

            // These may be unreachable - it depends if can_be_isize() passes
            Self::I64(v)       => Ok(v as isize),
            Self::I128(v)      => Ok(v as isize),

            // Easy!
            Self::ISize(v)     => Ok(v),
        }
    }

    /// Convert to a u128.
    ///
    /// Used for comparisons, arithmetic, etc.
    pub fn as_u128(self) -> u128 {
        match self {
            Self::U8(v)        => v as u128,
            Self::U16(v)       => v as u128,
            Self::U24(msb,lsb) => ((msb as u128)) << 16 | (lsb as u128),
            Self::U32(v)       => v as u128,
            Self::U64(v)       => v as u128,
            Self::U128(v)      => v as u128,
            Self::USize(v)     => v as u128,
            Self::I8(v)        => v as u128,
            Self::I16(v)       => v as u128,
            Self::I32(v)       => v as u128,
            Self::I64(v)       => v as u128,
            Self::I128(v)      => v as u128,
            Self::ISize(v)     => v as u128,
        }
    }

    /// Increment the value
    #[must_use] // So users don't assume this is mutable
    pub fn increment(self) -> Option<Self> {
        Some(match self {
            Self::U8(v)        => Self::from(v.checked_add(1)?),
            Self::U16(v)       => Self::from(v.checked_add(1)?),
            Self::U24(_,_) => {
                // We have to handle U24 specially, unfortunately
                let i = self.as_u128() + 1;
                if i > 0x00FFFFFF {
                    return None;
                }

                Self::from((
                        ((i as u32) >> 16 & 0x000000FF) as u8, // MSB
                        ((i as u32) & 0x0000FFFFu32) as u16,   // LSB
                ))
            },
            Self::U32(v)       => Self::from(v.checked_add(1)?),
            Self::U64(v)       => Self::from(v.checked_add(1)?),
            Self::U128(v)      => Self::from(v.checked_add(1)?),
            Self::USize(v)     => Self::from(v.checked_add(1)?),
            Self::I8(v)        => Self::from(v.checked_add(1)?),
            Self::I16(v)       => Self::from(v.checked_add(1)?),
            Self::I32(v)       => Self::from(v.checked_add(1)?),
            Self::I64(v)       => Self::from(v.checked_add(1)?),
            Self::I128(v)      => Self::from(v.checked_add(1)?),
            Self::ISize(v)     => Self::from(v.checked_add(1)?),
        })
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)    => fmt::Display::fmt(&v, f),
            Self::U16(v)   => fmt::Display::fmt(&v, f),
            Self::U24(msb,lsb) => fmt::Display::fmt(&(((*msb as u32) << 16) | (*lsb as u32)), f),
            Self::U32(v)   => fmt::Display::fmt(&v, f),
            Self::U64(v)   => fmt::Display::fmt(&v, f),
            Self::U128(v)  => fmt::Display::fmt(&v, f),
            Self::USize(v) => fmt::Display::fmt(&v, f),

            Self::I8(v)    => fmt::Display::fmt(&v, f),
            Self::I16(v)   => fmt::Display::fmt(&v, f),
            Self::I32(v)   => fmt::Display::fmt(&v, f),
            Self::I64(v)   => fmt::Display::fmt(&v, f),
            Self::I128(v)  => fmt::Display::fmt(&v, f),
            Self::ISize(v) => fmt::Display::fmt(&v, f),
        }
    }
}

impl fmt::LowerHex for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)    => fmt::LowerHex::fmt(&v, f),
            Self::U16(v)   => fmt::LowerHex::fmt(&v, f),
            Self::U24(msb,lsb) => fmt::LowerHex::fmt(&(((*msb as u32) << 16) | (*lsb as u32)), f),
            Self::U32(v)   => fmt::LowerHex::fmt(&v, f),
            Self::U64(v)   => fmt::LowerHex::fmt(&v, f),
            Self::U128(v)  => fmt::LowerHex::fmt(&v, f),
            Self::USize(v) => fmt::LowerHex::fmt(&v, f),

            Self::I8(v)    => fmt::LowerHex::fmt(&v, f),
            Self::I16(v)   => fmt::LowerHex::fmt(&v, f),
            Self::I32(v)   => fmt::LowerHex::fmt(&v, f),
            Self::I64(v)   => fmt::LowerHex::fmt(&v, f),
            Self::I128(v)  => fmt::LowerHex::fmt(&v, f),
            Self::ISize(v) => fmt::LowerHex::fmt(&v, f),
        }
    }
}

impl fmt::UpperHex for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)    => fmt::UpperHex::fmt(&v, f),
            Self::U16(v)   => fmt::UpperHex::fmt(&v, f),
            Self::U24(msb,lsb) => fmt::UpperHex::fmt(&(((*msb as u32) << 16) | (*lsb as u32)), f),
            Self::U32(v)   => fmt::UpperHex::fmt(&v, f),
            Self::U64(v)   => fmt::UpperHex::fmt(&v, f),
            Self::U128(v)  => fmt::UpperHex::fmt(&v, f),
            Self::USize(v) => fmt::UpperHex::fmt(&v, f),

            Self::I8(v)    => fmt::UpperHex::fmt(&v, f),
            Self::I16(v)   => fmt::UpperHex::fmt(&v, f),
            Self::I32(v)   => fmt::UpperHex::fmt(&v, f),
            Self::I64(v)   => fmt::UpperHex::fmt(&v, f),
            Self::I128(v)  => fmt::UpperHex::fmt(&v, f),
            Self::ISize(v) => fmt::UpperHex::fmt(&v, f),
        }
    }
}

impl fmt::Octal for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)    => fmt::Octal::fmt(&v, f),
            Self::U16(v)   => fmt::Octal::fmt(&v, f),
            Self::U24(msb,lsb) => fmt::Octal::fmt(&(((*msb as u32) << 16) | (*lsb as u32)), f),
            Self::U32(v)   => fmt::Octal::fmt(&v, f),
            Self::U64(v)   => fmt::Octal::fmt(&v, f),
            Self::U128(v)  => fmt::Octal::fmt(&v, f),
            Self::USize(v) => fmt::Octal::fmt(&v, f),

            Self::I8(v)    => fmt::Octal::fmt(&v, f),
            Self::I16(v)   => fmt::Octal::fmt(&v, f),
            Self::I32(v)   => fmt::Octal::fmt(&v, f),
            Self::I64(v)   => fmt::Octal::fmt(&v, f),
            Self::I128(v)  => fmt::Octal::fmt(&v, f),
            Self::ISize(v) => fmt::Octal::fmt(&v, f),
        }
    }
}

impl fmt::LowerExp for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)    => fmt::LowerExp::fmt(&v, f),
            Self::U16(v)   => fmt::LowerExp::fmt(&v, f),
            Self::U24(msb,lsb) => fmt::LowerExp::fmt(&(((*msb as u32) << 16) | (*lsb as u32)), f),
            Self::U32(v)   => fmt::LowerExp::fmt(&v, f),
            Self::U64(v)   => fmt::LowerExp::fmt(&v, f),
            Self::U128(v)  => fmt::LowerExp::fmt(&v, f),
            Self::USize(v) => fmt::LowerExp::fmt(&v, f),

            Self::I8(v)    => fmt::LowerExp::fmt(&v, f),
            Self::I16(v)   => fmt::LowerExp::fmt(&v, f),
            Self::I32(v)   => fmt::LowerExp::fmt(&v, f),
            Self::I64(v)   => fmt::LowerExp::fmt(&v, f),
            Self::I128(v)  => fmt::LowerExp::fmt(&v, f),
            Self::ISize(v) => fmt::LowerExp::fmt(&v, f),
        }
    }
}

impl fmt::UpperExp for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)    => fmt::UpperExp::fmt(&v, f),
            Self::U16(v)   => fmt::UpperExp::fmt(&v, f),
            Self::U24(msb,lsb) => fmt::UpperExp::fmt(&(((*msb as u32) << 16) | (*lsb as u32)), f),
            Self::U32(v)   => fmt::UpperExp::fmt(&v, f),
            Self::U64(v)   => fmt::UpperExp::fmt(&v, f),
            Self::U128(v)  => fmt::UpperExp::fmt(&v, f),
            Self::USize(v) => fmt::UpperExp::fmt(&v, f),

            Self::I8(v)    => fmt::UpperExp::fmt(&v, f),
            Self::I16(v)   => fmt::UpperExp::fmt(&v, f),
            Self::I32(v)   => fmt::UpperExp::fmt(&v, f),
            Self::I64(v)   => fmt::UpperExp::fmt(&v, f),
            Self::I128(v)  => fmt::UpperExp::fmt(&v, f),
            Self::ISize(v) => fmt::UpperExp::fmt(&v, f),
        }
    }
}

impl fmt::Binary for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v)    => fmt::Binary::fmt(&v, f),
            Self::U16(v)   => fmt::Binary::fmt(&v, f),
            Self::U24(msb,lsb) => fmt::Binary::fmt(&(((*msb as u32) << 16) | (*lsb as u32)), f),
            Self::U32(v)   => fmt::Binary::fmt(&v, f),
            Self::U64(v)   => fmt::Binary::fmt(&v, f),
            Self::U128(v)  => fmt::Binary::fmt(&v, f),
            Self::USize(v) => fmt::Binary::fmt(&v, f),

            Self::I8(v)    => fmt::Binary::fmt(&v, f),
            Self::I16(v)   => fmt::Binary::fmt(&v, f),
            Self::I32(v)   => fmt::Binary::fmt(&v, f),
            Self::I64(v)   => fmt::Binary::fmt(&v, f),
            Self::I128(v)  => fmt::Binary::fmt(&v, f),
            Self::ISize(v) => fmt::Binary::fmt(&v, f),
        }
    }
}

impl PartialEq for Integer {
    fn eq(&self, other: &Self) -> bool {
        self.as_u128().eq(&other.as_u128())
    }
}

impl Eq for Integer {
    // Automatically uses PartialEq
}

impl Hash for Integer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_u128().hash(state)
    }
}

// I decided not to implement PartialOrd at all - nothing was depending on it,
// and comparing signed with unsigned values was harrowingly weird. :)
// impl PartialOrd for Integer {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         // Try to compare as signed
//         if let Some(a) = self.as_i128() {
//             if let Some(b) = other.as_i128() {
//                 return a.partial_cmp(&b);
//             }
//         }

//         // If we can't compare as signed, compare as unsigned
//         self.as_u128().partial_cmp(&other.as_u128())
//     }
// }

impl FromStr for Integer {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        // Handle other types
        let (radix, s) = if s.len() > 2 {
            match &s[0..2] {
                "0b" => (2,  &s[2..]), // Binary
                "0o" => (8,  &s[2..]), // Octal
                "0x" => (16, &s[2..]), // Hex
                _    => (10, s),
            }
        } else {
            (10, s)
        };

        // Try usize/isize first
        let i = usize::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        let i = isize::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        // Try u64/i64
        let i = u64::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        let i = i64::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        // Try u64/i64
        let i = u64::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        let i = i64::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        // Not sure if we really need to, but try u128/i128
        let i = u128::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        let i = i128::from_str_radix(s, radix).map(|i| Integer::from(i));
        if let Ok(i) = i {
            return Ok(i)
        }

        // Give up
        bail!("String does not appear to be a valid integer: {}", s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::{assert_eq, assert_ne};
    use simple_error::SimpleResult;

    use crate::{Context, Integer, IntegerReader, IntegerRendererTrait, Endian, DefaultFormatter};

    #[test]
    fn test_display() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

        let formatter = DefaultFormatter::new();

        assert_eq!("0",    formatter.render_integer(IntegerReader::U8.read(Context::new_at(&data, 0))?));
        assert_eq!("127",  formatter.render_integer(IntegerReader::U8.read(Context::new_at(&data, 1))?));
        assert_eq!("128",  formatter.render_integer(IntegerReader::U8.read(Context::new_at(&data, 2))?));
        assert_eq!("255",  formatter.render_integer(IntegerReader::U8.read(Context::new_at(&data, 3))?));

        assert_eq!("0",    formatter.render_integer(IntegerReader::I8.read(Context::new_at(&data, 0))?));
        assert_eq!("127",  formatter.render_integer(IntegerReader::I8.read(Context::new_at(&data, 1))?));
        assert_eq!("-128", formatter.render_integer(IntegerReader::I8.read(Context::new_at(&data, 2))?));
        assert_eq!("-1",   formatter.render_integer(IntegerReader::I8.read(Context::new_at(&data, 3))?));


        Ok(())
    }

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

    #[test]
    fn test_comparison() -> SimpleResult<()> {
        // Unsigned -> unsigned
        assert_eq!(Integer::from(0u8), Integer::from(0u8));
        assert_eq!(Integer::from(0u8), Integer::from(0u16));
        assert_eq!(Integer::from(0u8), Integer::from(0u64));
        assert_eq!(Integer::from(0u8), Integer::from(0u128));

        // Signed -> signed
        assert_eq!(Integer::from(0i8), Integer::from(0i8));
        assert_eq!(Integer::from(0i8), Integer::from(0i16));
        assert_eq!(Integer::from(0i8), Integer::from(0i64));
        assert_eq!(Integer::from(0i8), Integer::from(0i128));

        // Signed -> unsigned should still be equal
        assert_eq!(Integer::from(0u8), Integer::from(0i8));
        assert_eq!(Integer::from(0u8), Integer::from(0i64));

        // We accept that signed/unsigned comparisons can be a bit weird
        assert_eq!(Integer::from(-1i8), Integer::from(0xffffffffffffffffffffffffffffffffu128));

        // But that shouldn't affect most comparisons
        // Note that these are NE!!
        assert_ne!(Integer::from(0x80u8), Integer::from(-128i8));
        assert_ne!(Integer::from(-1i16),  Integer::from(0xffffu16));

        // Ordering no longer works, it was too weird
        // assert!(Integer::from(0u8)  < Integer::from(1u32));
        // assert!(Integer::from(0u32) < Integer::from(1u8));
        // assert!(Integer::from(1u8)  > Integer::from(0u32));
        // assert!(Integer::from(1u32) > Integer::from(0u8));
        // assert!(Integer::from(-1i8) < Integer::from(1i8));

        Ok(())
    }

    #[test]
    fn test_u24() -> SimpleResult<()> {
        let data = b"\x01\x02\x03\x04\x05\x06".to_vec();

        let tests = vec![
            // offset     endian          expected
            (  0,         Endian::Big,    0x010203u32),
            (  1,         Endian::Big,    0x020304u32),
            (  2,         Endian::Big,    0x030405u32),
            (  3,         Endian::Big,    0x040506u32),

            (  0,         Endian::Little, 0x030201u32),
            (  1,         Endian::Little, 0x040302u32),
            (  2,         Endian::Little, 0x050403u32),
            (  3,         Endian::Little, 0x060504u32),
        ];

        for (offset, endian, expected) in tests {
            let c = Context::new_at(&data, offset);
            let reader = IntegerReader::U24(endian);

            assert_eq!(Integer::from(expected), reader.read(c)?);
        }

        Ok(())
    }

    #[test]
    fn test_from_str() -> SimpleResult<()> {
        let tests = vec![
            // string                  expected
            (  "1",                    Integer::from(1u8)), // <-- the Eq implementation means we can vary the types
            (  "1",                    Integer::from(1usize)),

            (  "-1",                   Integer::from(-1)),
            (  "-65535",               Integer::from(-65535)),

            (  "255",                  Integer::from(0xffusize)),
            (  "65535",                Integer::from(0xffffu32)),
            (  "18446744073709551615", Integer::from(0xffffffffffffffffu64)),
            (  "79228162514264337593543950335", Integer::from(0xffffffffffffffffffffffffu128)),

            (  "0xff",                 Integer::from(0xffu32)),
            (  "0xf",                  Integer::from(0xfu32)),

            (  "0o777",                Integer::from(511u32)),
            (  "0b1111",               Integer::from(15u32)),

        ];

        for (s, expected) in tests {
            assert_eq!(Integer::from_str(s)?, expected);
        }

        Ok(())
    }

    #[test]
    fn test_increment() -> SimpleResult<()> {
        // Test normal stuff
        let i = Integer::from(1u8);
        assert_eq!(Some(Integer::from(2u8)), i.increment());

        // Test overflow
        let i = Integer::from(255u8);
        assert_eq!(None, i.increment());

        // Test u24, because it's special
        let i = Integer::from((0x00u8, 0x0000u16));
        assert_eq!(Some(Integer::from(0x000001u32)), i.increment());

        // Just a middling u24 value
        let i = Integer::from((0x12u8, 0x1234u16));
        assert_eq!(Some(Integer::from(0x121235u32)), i.increment());

        // Test first overflow
        let i = Integer::from((0x00u8, 0xFFFFu16));
        assert_eq!(Some(Integer::from(0x010000u32)), i.increment());

        // But what about second overflow?
        let i = Integer::from((0xFFu8, 0xFFFFu16));
        assert_eq!(None, i.increment());

        Ok(())
    }
}
