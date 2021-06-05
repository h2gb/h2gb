use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::generic_number::{Context, Endian, GenericNumber};

/// Define how data is read from a Context.
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
    /// Note: floats can only be displayed as `GenericFormatter::Decimal` or
    /// `GenericFormatter::Scientific`.
    F32(Endian),

    /// Signed 64-bit (aka, double precision) floating point
    ///
    /// Note: floats can only be displayed as `GenericFormatter::Decimal` or
    /// `GenericFormatter::Scientific`.
    F64(Endian),
}

impl GenericReader {
    pub fn read(self, context: Context) -> SimpleResult<GenericNumber> {
        match self {
            Self::I8           => Ok(GenericNumber::from(context.read_i8()?)),
            Self::I16(endian)  => Ok(GenericNumber::from(context.read_i16(endian)?)),
            Self::I32(endian)  => Ok(GenericNumber::from(context.read_i32(endian)?)),
            Self::I64(endian)  => Ok(GenericNumber::from(context.read_i64(endian)?)),
            Self::I128(endian) => Ok(GenericNumber::from(context.read_i128(endian)?)),

            Self::U8           => Ok(GenericNumber::from(context.read_u8()?)),
            Self::U16(endian)  => Ok(GenericNumber::from(context.read_u16(endian)?)),
            Self::U32(endian)  => Ok(GenericNumber::from(context.read_u32(endian)?)),
            Self::U64(endian)  => Ok(GenericNumber::from(context.read_u64(endian)?)),
            Self::U128(endian) => Ok(GenericNumber::from(context.read_u128(endian)?)),

            Self::F32(endian)  => Ok(GenericNumber::from(context.read_f32(endian)?)),
            Self::F64(endian)  => Ok(GenericNumber::from(context.read_f64(endian)?)),
        }
    }

    pub fn size(self) -> usize {
        match self {
            Self::I8      => 1,
            Self::I16(_)  => 2,
            Self::I32(_)  => 4,
            Self::I64(_)  => 8,
            Self::I128(_) => 16,

            Self::U8      => 1,
            Self::U16(_)  => 2,
            Self::U32(_)  => 4,
            Self::U64(_)  => 8,
            Self::U128(_) => 16,

            Self::F32(_)  => 4,
            Self::F64(_)  => 8,
        }
    }

    pub fn can_be_u64(self) -> bool {
        match self {
            Self::I8      => false,
            Self::I16(_)  => false,
            Self::I32(_)  => false,
            Self::I64(_)  => false,
            Self::I128(_) => false,

            Self::U8      => true,
            Self::U16(_)  => true,
            Self::U32(_)  => true,
            Self::U64(_)  => true,
            Self::U128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }

    pub fn can_be_i64(self) -> bool {
        match self {
            Self::I8      => true,
            Self::I16(_)  => true,
            Self::I32(_)  => true,
            Self::I64(_)  => true,
            Self::I128(_) => false,

            Self::U8      => false,
            Self::U16(_)  => false,
            Self::U32(_)  => false,
            Self::U64(_)  => false,
            Self::U128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use pretty_assertions::assert_eq;
//     use simple_error::SimpleResult;

//     #[test]
//     fn test_buffer_too_short() -> SimpleResult<()> {
//         let data = b"".to_vec();
//         assert!(GenericReader::I8.to_string(Context::new(&data), GenericFormatter::Decimal(Default::default())).is_err());

//         let data = b"A".to_vec();
//         assert!(GenericReader::I16(Endian::Big).to_string(Context::new(&data), GenericFormatter::Decimal(Default::default())).is_err());

//         let data = b"AAA".to_vec();
//         assert!(GenericReader::I32(Endian::Big).to_string(Context::new(&data), GenericFormatter::Decimal(Default::default())).is_err());

//         let data = b"AAAAAAA".to_vec();
//         assert!(GenericReader::I64(Endian::Big).to_string(Context::new(&data), GenericFormatter::Decimal(Default::default())).is_err());

//         let data = b"AAAAAAAAAAAAAAA".to_vec();
//         assert!(GenericReader::I128(Endian::Big).to_string(Context::new(&data), GenericFormatter::Decimal(Default::default())).is_err());

//         Ok(())
//     }

//     #[test]
//     fn test_to_u64() -> SimpleResult<()> {
//         let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03".to_vec();

//         assert_eq!(0u64,   GenericReader::U8.to_u64(Context::new_at(&data, 0))?);
//         assert_eq!(127u64, GenericReader::U8.to_u64(Context::new_at(&data, 1))?);
//         assert_eq!(128u64, GenericReader::U8.to_u64(Context::new_at(&data, 2))?);
//         assert_eq!(255u64, GenericReader::U8.to_u64(Context::new_at(&data, 3))?);

//         assert_eq!(127u64,               GenericReader::U16(Endian::Big).to_u64(Context::new_at(&data, 0))?);
//         assert_eq!(8356095u64,           GenericReader::U32(Endian::Big).to_u64(Context::new_at(&data, 0))?);
//         assert_eq!(35889154747335171u64, GenericReader::U64(Endian::Big).to_u64(Context::new_at(&data, 0))?);

//         assert!(GenericReader::U128(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::I8.to_u64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::I16(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::I32(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::I64(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::F32(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::F64(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());

//         Ok(())
//     }

//     #[test]
//     fn test_to_i64() -> SimpleResult<()> {
//         let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

//         assert_eq!(0i64,                    GenericReader::I8.to_i64(Context::new_at(&data, 0))?);
//         assert_eq!(127i64,                  GenericReader::I8.to_i64(Context::new_at(&data, 1))?);
//         assert_eq!(-128i64,                 GenericReader::I8.to_i64(Context::new_at(&data, 2))?);
//         assert_eq!(-1i64,                   GenericReader::I8.to_i64(Context::new_at(&data, 3))?);

//         assert_eq!(127i64,                  GenericReader::I16(Endian::Big).to_i64(Context::new_at(&data, 0))?);
//         assert_eq!(-32768i64,               GenericReader::I16(Endian::Big).to_i64(Context::new_at(&data, 8))?);

//         assert_eq!(8356095i64,              GenericReader::I32(Endian::Big).to_i64(Context::new_at(&data, 0))?);
//         assert_eq!(-2147483648i64,          GenericReader::I32(Endian::Big).to_i64(Context::new_at(&data, 8))?);

//         assert_eq!(35889154747335171i64,    GenericReader::I64(Endian::Big).to_i64(Context::new_at(&data, 0))?);
//         assert_eq!(-9223372036854775808i64, GenericReader::I64(Endian::Big).to_i64(Context::new_at(&data, 8))?);

//         assert!(GenericReader::I128(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::U8.to_i64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::U16(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::U32(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::U64(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::F32(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
//         assert!(GenericReader::F64(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());

//         Ok(())
//     }
// }
