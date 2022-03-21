use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;

use crate::{H2TypeTrait, Data};
use crate::simple::*;
use crate::simple::network::*;
use crate::simple::numeric::*;
use crate::simple::string::*;
use crate::composite::*;

/// The core of this crate - defines any type of value abstractly.
///
/// In general, when consuming this crate, you probably won't be creating an
/// `H2Type` directly; rather, create one of the [`crate::simple`] or
/// [`crate::composite`] types, then use `.into()` to get H2Type.
///
/// Please note that many of the functions here are very expensive, because
/// they have to read the object and iterate every time they're called. If you
/// call `resolve()`, a static version will be created with the fields pre-
/// calculated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2Type {
    // Simple
    //H2Pointer(H2Pointer),
    Rgb(Rgb),
    H2Bitmask(H2Bitmask),
    H2Enum(H2Enum),
    H2UUID(H2UUID),
    H2Blob(H2Blob),

    // Numeric
    H2Character(H2Character),
    H2Float(H2Float),
    H2Integer(H2Integer),

    // Network
    IPv4(IPv4),
    IPv6(IPv6),
    MacAddress(MacAddress),
    MacAddress8(MacAddress8),

    // Strings
    H2String(H2String),
    NTString(NTString),
    LPString(LPString),

    // Composite
    H2Array(H2Array),
    H2Struct(H2Struct),

}

impl H2Type {
    pub fn as_trait(&self, _data: &Data) -> SimpleResult<&dyn H2TypeTrait> {
        Ok(match self {
            // Simple
            H2Type::Rgb(t)       => t,
            H2Type::H2Bitmask(t) => t,
            H2Type::H2Enum(t)    => t,
            H2Type::H2UUID(t)    => t,
            H2Type::H2Blob(t)    => t,

            // Numeric
            H2Type::H2Float(t)     => t,
            H2Type::H2Character(t) => t,
            H2Type::H2Integer(t)   => t,

            // Network
            H2Type::IPv4(t)        => t,
            H2Type::IPv6(t)        => t,
            H2Type::MacAddress(t)  => t,
            H2Type::MacAddress8(t) => t,

            // Complex
            H2Type::H2Array(t)   => t,
            H2Type::H2Struct(t)  => t,

            // Strings
            H2Type::H2String(t)  => t,
            H2Type::NTString(t)  => t,
            H2Type::LPString(t)  => t,
        })
    }
}
