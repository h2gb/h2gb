use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
}

// Simplify converting from various basic types
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

impl GenericNumber {
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
        }
    }

    pub fn can_be_u64(self) -> bool {
        match self {
            Self::U8(_)   => true,
            Self::U16(_)  => true,
            Self::U32(_)  => true,
            Self::U64(_)  => true,
            Self::U128(_) => false,

            Self::I8(_)   => false,
            Self::I16(_)  => false,
            Self::I32(_)  => false,
            Self::I64(_)  => false,
            Self::I128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }

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
        }
    }

    pub fn can_be_i64(self) -> bool {
        match self {
            Self::U8(_)   => false,
            Self::U16(_)  => false,
            Self::U32(_)  => false,
            Self::U64(_)  => false,
            Self::U128(_) => false,

            Self::I8(_)   => true,
            Self::I16(_)  => true,
            Self::I32(_)  => true,
            Self::I64(_)  => true,
            Self::I128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }

    pub fn as_i64(self) -> SimpleResult<i64> {
        match self {
            Self::I8(v)        => Ok(v as i64),
            Self::I16(v)       => Ok(v as i64),
            Self::I32(v)       => Ok(v as i64),
            Self::I64(v)       => Ok(v as i64),

            // None of these can become i32
            Self::I128(_) => bail!("Can't convert i128 into i64"),

            Self::U8(_)   => bail!("Can't convert u8 (unsigned) into i64"),
            Self::U16(_)  => bail!("Can't convert u16 (unsigned) into i64"),
            Self::U32(_)  => bail!("Can't convert u32 (unsigned) into i64"),
            Self::U64(_)  => bail!("Can't convert u64 (unsigned) into i64"),
            Self::U128(_) => bail!("Can't convert u128 into i64"),
            Self::F32(_)  => bail!("Can't convert floating point into i64"),
            Self::F64(_)  => bail!("Can't convert floating point into i64"),
        }
    }
}
