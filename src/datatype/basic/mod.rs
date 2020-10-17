use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use sized_number::Context;

use crate::datatype::H2StaticType;
use crate::datatype::composite::h2simple::H2Simple;

pub mod h2number;
use h2number::H2Number;

pub mod h2pointer;
use h2pointer::H2Pointer;

pub mod ipv4;
use ipv4::IPv4;

pub mod ipv6;
use ipv6::IPv6;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2BasicType {
    Number(H2Number),
    Pointer(H2Pointer),
    IPv4(IPv4),
    IPv6(IPv6),
}

impl From<H2BasicType> for H2StaticType {
    fn from(o: H2BasicType) -> H2StaticType {
        H2StaticType::from(H2Simple::new(o))
    }
}

impl H2BasicType {
    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        match self {
            Self::Number(t)  => t.to_string(context),
            Self::Pointer(t) => t.to_string(context),
            Self::IPv4(t)    => t.to_string(context),
            Self::IPv6(t)    => t.to_string(context),
        }
    }

    pub fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2StaticType)>> {
        match self {
            Self::Number(t)  => t.related(context),
            Self::Pointer(t) => t.related(context),
            Self::IPv4(t)    => t.related(context),
            Self::IPv6(t)    => t.related(context),
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            Self::Number(t)  => t.size(),
            Self::Pointer(t) => t.size(),
            Self::IPv4(t)    => t.size(),
            Self::IPv6(t)    => t.size(),
        }
    }
}
