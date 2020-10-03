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
pub enum NumberDisplayFormat {
    Hex,
    HexUppercase,
    Decimal,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Endian {
    BigEndian,
    LittleEndian,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NumberFormat {
    size: NumberSize,
    display: NumberDisplayFormat,
    endian: Endian,
}

impl NumberFormat {
    pub const U8:  Self = Self {
        display: NumberDisplayFormat::Hex,
        size: NumberSize::Eight,
        endian: Endian::BigEndian,
    };

    pub const U16_BIG:  Self = Self {
        display: NumberDisplayFormat::Hex,
        size: NumberSize::Sixteen,
        endian: Endian::BigEndian,
    };

    pub const U32_BIG:  Self = Self {
        display: NumberDisplayFormat::Hex,
        size: NumberSize::ThirtyTwo,
        endian: Endian::BigEndian,
    };

    pub const U64_BIG:  Self = Self {
        display: NumberDisplayFormat::Hex,
        size: NumberSize::SixtyFour,
        endian: Endian::BigEndian,
    };

    pub const U16_LITTLE:  Self = Self {
        display: NumberDisplayFormat::Hex,
        size: NumberSize::Sixteen,
        endian: Endian::LittleEndian,
    };

    pub const U32_LITTLE:  Self = Self {
        display: NumberDisplayFormat::Hex,
        size: NumberSize::ThirtyTwo,
        endian: Endian::LittleEndian,
    };

    pub const U64_LITTLE:  Self = Self {
        display: NumberDisplayFormat::Hex,
        size: NumberSize::SixtyFour,
        endian: Endian::LittleEndian,
    };

    //pub fn from_context(context: &H2Context, size: NumberSize, endian: Endian) -> std::io::Result<Self> {
    pub fn to_sized_number(self, context: &H2Context) -> SimpleResult<SizedNumber> {
        match SizedNumber::from_context(context, self.size, self.endian) {
            Ok(n) => Ok(n),
            Err(e) => bail!("Couldn't read number: {}", e),
        }
    }

    pub fn to_string(self, context: &H2Context) -> SimpleResult<String> {
        let number = self.to_sized_number(context)?;

        Ok(number.to_string(self.display))
    }

    pub fn size(self) -> u64 {
        self.size.size()
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

impl From<(NumberSize, u64)> for SizedNumber {
    fn from(o: (NumberSize, u64)) -> SizedNumber {
        match o.0 {
            NumberSize::Eight     => (o.1 as u8).into(),
            NumberSize::Sixteen   => (o.1 as u16).into(),
            NumberSize::ThirtyTwo => (o.1 as u32).into(),
            NumberSize::SixtyFour => (o.1 as u64).into(),
        }
    }
}

impl SizedNumber {
    pub fn from_context(context: &H2Context, size: NumberSize, endian: Endian) -> std::io::Result<Self> {
        let mut context = context.clone();

        Ok(match (size, endian) {
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

    pub fn to_string(self, format: NumberDisplayFormat) -> String {
        match (format, self) {
            (NumberDisplayFormat::Hex, Self::Eight(i))     => format!("{:02x}", i),
            (NumberDisplayFormat::Hex, Self::Sixteen(i))   => format!("{:04x}", i),
            (NumberDisplayFormat::Hex, Self::ThirtyTwo(i)) => format!("{:08x}", i),
            (NumberDisplayFormat::Hex, Self::SixtyFour(i)) => format!("{:016x}", i),

            (NumberDisplayFormat::Hex, Self::EightSigned(i))     => format!("{:02x}", i),
            (NumberDisplayFormat::Hex, Self::SixteenSigned(i))   => format!("{:04x}", i),
            (NumberDisplayFormat::Hex, Self::ThirtyTwoSigned(i)) => format!("{:08x}", i),
            (NumberDisplayFormat::Hex, Self::SixtyFourSigned(i)) => format!("{:016x}", i),

            (NumberDisplayFormat::HexUppercase, Self::Eight(i))     => format!("{:02X}", i),
            (NumberDisplayFormat::HexUppercase, Self::Sixteen(i))   => format!("{:04X}", i),
            (NumberDisplayFormat::HexUppercase, Self::ThirtyTwo(i)) => format!("{:08X}", i),
            (NumberDisplayFormat::HexUppercase, Self::SixtyFour(i)) => format!("{:016X}", i),

            (NumberDisplayFormat::HexUppercase, Self::EightSigned(i))     => format!("{:02X}", i),
            (NumberDisplayFormat::HexUppercase, Self::SixteenSigned(i))   => format!("{:04X}", i),
            (NumberDisplayFormat::HexUppercase, Self::ThirtyTwoSigned(i)) => format!("{:08X}", i),
            (NumberDisplayFormat::HexUppercase, Self::SixtyFourSigned(i)) => format!("{:016X}", i),

            (NumberDisplayFormat::Decimal, Self::Eight(i))     => format!("{}", i),
            (NumberDisplayFormat::Decimal, Self::Sixteen(i))   => format!("{}", i),
            (NumberDisplayFormat::Decimal, Self::ThirtyTwo(i)) => format!("{}", i),
            (NumberDisplayFormat::Decimal, Self::SixtyFour(i)) => format!("{}", i),

            (NumberDisplayFormat::Decimal, Self::EightSigned(i))     => format!("{}", i),
            (NumberDisplayFormat::Decimal, Self::SixteenSigned(i))   => format!("{}", i),
            (NumberDisplayFormat::Decimal, Self::ThirtyTwoSigned(i)) => format!("{}", i),
            (NumberDisplayFormat::Decimal, Self::SixtyFourSigned(i)) => format!("{}", i),
        }
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
