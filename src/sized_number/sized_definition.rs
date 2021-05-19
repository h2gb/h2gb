use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::sized_number::{Context, Endian, SizedDisplay};

/// Define how data is read from a Context.
///
/// This is the core of `sized_number` - it's how the numbers are defined in
/// memory.
///
/// The options all pretty cleanly map to the equivalent datatypes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SizedDefinition {
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
    /// Note: floats can only be displayed as `SizedDisplay::Decimal` or
    /// `SizedDisplay::Scientific`.
    F32(Endian),

    /// Signed 64-bit (aka, double precision) floating point
    ///
    /// Note: floats can only be displayed as `SizedDisplay::Decimal` or
    /// `SizedDisplay::Scientific`.
    F64(Endian),
}

impl SizedDefinition {
    /// Returns the size, in bytes, of the current type.
    pub fn size(self) -> u64 {
        match self {
            Self::U8      => 1,
            Self::U16(_)  => 2,
            Self::U32(_)  => 4,
            Self::U64(_)  => 8,
            Self::U128(_) => 16,

            Self::I8      => 1,
            Self::I16(_)  => 2,
            Self::I32(_)  => 4,
            Self::I64(_)  => 8,
            Self::I128(_) => 16,

            Self::F32(_)  => 4,
            Self::F64(_)  => 8,
        }
    }

    /// Read data from the context, based on the [`SizedDefinition`], and
    /// display it based on the `SizedDisplay`
    pub fn to_string(self, context: Context, display: SizedDisplay) -> SimpleResult<String> {
        let options = display.to_options();

        match self {
            Self::I8           => options.to_s_i8(context.read_i8()?),
            Self::I16(endian)  => options.to_s_i16(context.read_i16(endian)?),
            Self::I32(endian)  => options.to_s_i32(context.read_i32(endian)?),
            Self::I64(endian)  => options.to_s_i64(context.read_i64(endian)?),
            Self::I128(endian) => options.to_s_i128(context.read_i128(endian)?),

            Self::U8           => options.to_s_u8(context.read_u8()?),
            Self::U16(endian)  => options.to_s_u16(context.read_u16(endian)?),
            Self::U32(endian)  => options.to_s_u32(context.read_u32(endian)?),
            Self::U64(endian)  => options.to_s_u64(context.read_u64(endian)?),
            Self::U128(endian) => options.to_s_u128(context.read_u128(endian)?),

            Self::F32(endian)  => options.to_s_f32(context.read_f32(endian)?),
            Self::F64(endian)  => options.to_s_f64(context.read_f64(endian)?),
        }
    }

    /// Returns `true` for variants that can succeed in [`to_u64`]
    pub fn can_be_u64(self) -> bool {
        match self {
            Self::U8      => true,
            Self::U16(_)  => true,
            Self::U32(_)  => true,
            Self::U64(_)  => true,
            Self::U128(_) => false,

            Self::I8      => false,
            Self::I16(_)  => false,
            Self::I32(_)  => false,
            Self::I64(_)  => false,
            Self::I128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }

    /// Returns `true` for variants that can succeed in [`to_i64`]
    pub fn can_be_i64(self) -> bool {
        match self {
            Self::U8      => false,
            Self::U16(_)  => false,
            Self::U32(_)  => false,
            Self::U64(_)  => false,
            Self::U128(_) => false,

            Self::I8      => true,
            Self::I16(_)  => true,
            Self::I32(_)  => true,
            Self::I64(_)  => true,
            Self::I128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }

    /// Returns `true` for variants that can be displayed as an Enum
    pub fn can_be_enum(self) -> bool {
        self.can_be_u64() || self.can_be_i64()
    }

    /// Convert to an unsigned 64-bit value, if possible.
    ///
    /// Only unsigned values of 64-bits or less can be converted to a [`u64`].
    /// Everything else will return an error - we don't typecast signed to
    /// unsigned.
    pub fn to_u64(self, context: Context) -> SimpleResult<u64> {
        match self {
            Self::U8          => Ok(context.read_u8()? as u64),
            Self::U16(endian) => Ok(context.read_u16(endian)? as u64),
            Self::U32(endian) => Ok(context.read_u32(endian)? as u64),
            Self::U64(endian) => Ok(context.read_u64(endian)? as u64),

            // None of these can become u32
            Self::U128(_) => bail!("Can't convert u128 into u64"),

            Self::I8      => bail!("Can't convert i8 (signed) into u64"),
            Self::I16(_)  => bail!("Can't convert i16 (signed) into u64"),
            Self::I32(_)  => bail!("Can't convert i32 (signed) into u64"),
            Self::I64(_)  => bail!("Can't convert i64 (signed) into u64"),
            Self::I128(_) => bail!("Can't convert i128 (signed) into u64"),

            Self::F32(_)  => bail!("Can't convert floating point into u64"),
            Self::F64(_)  => bail!("Can't convert floating point into u64"),
        }
    }

    /// Convert to a signed 64-bit value, if possible.
    ///
    /// This will correctly extend the sign. So, for example, reading a
    /// `SizedDefinition::I8` with a value of `FF` will convert to the [`i64`]
    /// value `-1`, or `0xFFFFFFFFFFFFFFFF`.
    ///
    /// Only signed values of 64-bits or less can be converted to an [`i64`].
    /// Everything else will return an error - we don't typecast unsigned to
    /// signed.
    pub fn to_i64(self, context: Context) -> SimpleResult<i64> {
        match self {
            // Don't let unsigned values become signed
            Self::U8      => bail!("Can't convert i8 (signed) into i64"),
            Self::U16(_)  => bail!("Can't convert i16 (signed) into i64"),
            Self::U32(_)  => bail!("Can't convert i32 (signed) into i64"),
            Self::U64(_)  => bail!("Can't convert i64 (signed) into i64"),
            Self::U128(_) => bail!("Can't convert i128 (signed) into i64"),

            Self::I8 =>          Ok(context.read_i8()? as i64),
            Self::I16(endian) => Ok(context.read_i16(endian)? as i64),
            Self::I32(endian) => Ok(context.read_i32(endian)? as i64),
            Self::I64(endian) => Ok(context.read_i64(endian)? as i64),

            // 128 bit can't go into 64 bit
            Self::I128(_) => bail!("Can't convert u128 into i64"),

            // Float certainly can't
            Self::F32(_)  => bail!("Can't convert floating point into i64"),
            Self::F64(_)  => bail!("Can't convert floating point into i64"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    #[test]
    fn test_buffer_too_short() -> SimpleResult<()> {
        let data = b"".to_vec();
        assert!(SizedDefinition::I8.to_string(Context::new(&data), SizedDisplay::Decimal(Default::default())).is_err());

        let data = b"A".to_vec();
        assert!(SizedDefinition::I16(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal(Default::default())).is_err());

        let data = b"AAA".to_vec();
        assert!(SizedDefinition::I32(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal(Default::default())).is_err());

        let data = b"AAAAAAA".to_vec();
        assert!(SizedDefinition::I64(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal(Default::default())).is_err());

        let data = b"AAAAAAAAAAAAAAA".to_vec();
        assert!(SizedDefinition::I128(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal(Default::default())).is_err());

        Ok(())
    }

    #[test]
    fn test_to_u64() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03".to_vec();

        assert_eq!(0u64,   SizedDefinition::U8.to_u64(Context::new_at(&data, 0))?);
        assert_eq!(127u64, SizedDefinition::U8.to_u64(Context::new_at(&data, 1))?);
        assert_eq!(128u64, SizedDefinition::U8.to_u64(Context::new_at(&data, 2))?);
        assert_eq!(255u64, SizedDefinition::U8.to_u64(Context::new_at(&data, 3))?);

        assert_eq!(127u64,               SizedDefinition::U16(Endian::Big).to_u64(Context::new_at(&data, 0))?);
        assert_eq!(8356095u64,           SizedDefinition::U32(Endian::Big).to_u64(Context::new_at(&data, 0))?);
        assert_eq!(35889154747335171u64, SizedDefinition::U64(Endian::Big).to_u64(Context::new_at(&data, 0))?);

        assert!(SizedDefinition::U128(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I8.to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I16(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I32(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I64(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F32(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F64(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());

        Ok(())
    }

    #[test]
    fn test_to_i64() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

        assert_eq!(0i64,                    SizedDefinition::I8.to_i64(Context::new_at(&data, 0))?);
        assert_eq!(127i64,                  SizedDefinition::I8.to_i64(Context::new_at(&data, 1))?);
        assert_eq!(-128i64,                 SizedDefinition::I8.to_i64(Context::new_at(&data, 2))?);
        assert_eq!(-1i64,                   SizedDefinition::I8.to_i64(Context::new_at(&data, 3))?);

        assert_eq!(127i64,                  SizedDefinition::I16(Endian::Big).to_i64(Context::new_at(&data, 0))?);
        assert_eq!(-32768i64,               SizedDefinition::I16(Endian::Big).to_i64(Context::new_at(&data, 8))?);

        assert_eq!(8356095i64,              SizedDefinition::I32(Endian::Big).to_i64(Context::new_at(&data, 0))?);
        assert_eq!(-2147483648i64,          SizedDefinition::I32(Endian::Big).to_i64(Context::new_at(&data, 8))?);

        assert_eq!(35889154747335171i64,    SizedDefinition::I64(Endian::Big).to_i64(Context::new_at(&data, 0))?);
        assert_eq!(-9223372036854775808i64, SizedDefinition::I64(Endian::Big).to_i64(Context::new_at(&data, 8))?);

        assert!(SizedDefinition::I128(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U8.to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U16(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U32(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U64(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F32(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F64(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());

        Ok(())
    }
}
