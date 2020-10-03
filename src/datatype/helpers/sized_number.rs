use serde::{Serialize, Deserialize};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use simple_error::{SimpleResult, bail};

use crate::datatype::helpers::H2Context;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum NumberSize {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl NumberSize {
    pub fn size(self) -> u64 {
        match self {
            Self::Eight     => 1,
            Self::Sixteen   => 2,
            Self::ThirtyTwo => 4,
            Self::SixtyFour => 8,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Endian {
    BigEndian,
    LittleEndian,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct SizedFormat {
    size: NumberSize,
    endian: Endian,
}

impl SizedFormat {
    pub const U8:  Self = Self {
        size: NumberSize::Eight,
        endian: Endian::BigEndian,
    };

    pub const U16_BIG:  Self = Self {
        size: NumberSize::Sixteen,
        endian: Endian::BigEndian,
    };

    pub const U32_BIG:  Self = Self {
        size: NumberSize::ThirtyTwo,
        endian: Endian::BigEndian,
    };

    pub const U64_BIG:  Self = Self {
        size: NumberSize::SixtyFour,
        endian: Endian::BigEndian,
    };

    pub const U16_LITTLE:  Self = Self {
        size: NumberSize::Sixteen,
        endian: Endian::LittleEndian,
    };

    pub const U32_LITTLE:  Self = Self {
        size: NumberSize::ThirtyTwo,
        endian: Endian::LittleEndian,
    };

    pub const U64_LITTLE:  Self = Self {
        size: NumberSize::SixtyFour,
        endian: Endian::LittleEndian,
    };

    pub fn size(self) -> u64 {
        self.size.size()
    }

    pub fn read(self, context: &H2Context) -> SimpleResult<SizedNumber> {
        match SizedNumber::read(context, self) {
            Ok(n) => Ok(n),
            Err(e) => bail!("Couldn't read number: {}", e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SizedNumber {
    Eight(u8),
    Sixteen(u16),
    ThirtyTwo(u32),
    SixtyFour(u64),

    EightSigned(i8),
    SixteenSigned(i16),
    ThirtyTwoSigned(i32),
    SixtyFourSigned(i64),
}

impl From<u8> for SizedNumber {
    fn from(o: u8) -> SizedNumber {
        Self::Eight(o)
    }
}

impl From<u16> for SizedNumber {
    fn from(o: u16) -> SizedNumber {
        Self::Sixteen(o)
    }
}

impl From<u32> for SizedNumber {
    fn from(o: u32) -> SizedNumber {
        Self::ThirtyTwo(o)
    }
}

impl From<u64> for SizedNumber {
    fn from(o: u64) -> SizedNumber {
        Self::SixtyFour(o)
    }
}

impl From<i8> for SizedNumber {
    fn from(o: i8) -> SizedNumber {
        Self::EightSigned(o)
    }
}

impl From<i16> for SizedNumber {
    fn from(o: i16) -> SizedNumber {
        Self::SixteenSigned(o)
    }
}

impl From<i32> for SizedNumber {
    fn from(o: i32) -> SizedNumber {
        Self::ThirtyTwoSigned(o)
    }
}

impl From<i64> for SizedNumber {
    fn from(o: i64) -> SizedNumber {
        Self::SixtyFourSigned(o)
    }
}

impl SizedNumber {
    pub fn read(context: &H2Context, format: SizedFormat) -> std::io::Result<Self> {
        let mut context = context.clone();

        Ok(match (format.size, format.endian) {
            (NumberSize::Eight,     Endian::BigEndian)    => Self::from(context.read_u8()?),
            (NumberSize::Sixteen,   Endian::BigEndian)    => Self::from(context.read_u16::<BigEndian>()?),
            (NumberSize::ThirtyTwo, Endian::BigEndian)    => Self::from(context.read_u32::<BigEndian>()?),
            (NumberSize::SixtyFour, Endian::BigEndian)    => Self::from(context.read_u64::<BigEndian>()?),

            (NumberSize::Eight,     Endian::LittleEndian) => Self::from(context.read_u8()?),
            (NumberSize::Sixteen,   Endian::LittleEndian) => Self::from(context.read_u16::<LittleEndian>()?),
            (NumberSize::ThirtyTwo, Endian::LittleEndian) => Self::from(context.read_u32::<LittleEndian>()?),
            (NumberSize::SixtyFour, Endian::LittleEndian) => Self::from(context.read_u64::<LittleEndian>()?),


        })
    }

    pub fn to_index(self) -> SimpleResult<u64> {
        match self {
            Self::Eight(i)     => Ok(i as u64),
            Self::Sixteen(i)   => Ok(i as u64),
            Self::ThirtyTwo(i) => Ok(i as u64),
            Self::SixtyFour(i) => Ok(i as u64),

            Self::EightSigned(_i)     => bail!("Can't use a signed integer as an index (yet)"),
            Self::SixteenSigned(_i)   => bail!("Can't use a signed integer as an index (yet)"),
            Self::ThirtyTwoSigned(_i) => bail!("Can't use a signed integer as an index (yet)"),
            Self::SixtyFourSigned(_i) => bail!("Can't use a signed integer as an index (yet)"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SizedDisplay {
    Hex,
    HexUppercase,
    Decimal,
}

impl SizedDisplay {
    pub fn to_string(self, n: SizedNumber) -> String {
        match (self, n) {
            (Self::Hex, SizedNumber::Eight(i))     => format!("{:02x}", i),
            (Self::Hex, SizedNumber::Sixteen(i))   => format!("{:04x}", i),
            (Self::Hex, SizedNumber::ThirtyTwo(i)) => format!("{:08x}", i),
            (Self::Hex, SizedNumber::SixtyFour(i)) => format!("{:016x}", i),

            (Self::Hex, SizedNumber::EightSigned(i))     => format!("{:02x}", i),
            (Self::Hex, SizedNumber::SixteenSigned(i))   => format!("{:04x}", i),
            (Self::Hex, SizedNumber::ThirtyTwoSigned(i)) => format!("{:08x}", i),
            (Self::Hex, SizedNumber::SixtyFourSigned(i)) => format!("{:016x}", i),

            (Self::HexUppercase, SizedNumber::Eight(i))     => format!("{:02X}", i),
            (Self::HexUppercase, SizedNumber::Sixteen(i))   => format!("{:04X}", i),
            (Self::HexUppercase, SizedNumber::ThirtyTwo(i)) => format!("{:08X}", i),
            (Self::HexUppercase, SizedNumber::SixtyFour(i)) => format!("{:016X}", i),

            (Self::HexUppercase, SizedNumber::EightSigned(i))     => format!("{:02X}", i),
            (Self::HexUppercase, SizedNumber::SixteenSigned(i))   => format!("{:04X}", i),
            (Self::HexUppercase, SizedNumber::ThirtyTwoSigned(i)) => format!("{:08X}", i),
            (Self::HexUppercase, SizedNumber::SixtyFourSigned(i)) => format!("{:016X}", i),

            (Self::Decimal, SizedNumber::Eight(i))     => format!("{}", i),
            (Self::Decimal, SizedNumber::Sixteen(i))   => format!("{}", i),
            (Self::Decimal, SizedNumber::ThirtyTwo(i)) => format!("{}", i),
            (Self::Decimal, SizedNumber::SixtyFour(i)) => format!("{}", i),

            (Self::Decimal, SizedNumber::EightSigned(i))     => format!("{}", i),
            (Self::Decimal, SizedNumber::SixteenSigned(i))   => format!("{}", i),
            (Self::Decimal, SizedNumber::ThirtyTwoSigned(i)) => format!("{}", i),
            (Self::Decimal, SizedNumber::SixtyFourSigned(i)) => format!("{}", i),
        }
    }
}
