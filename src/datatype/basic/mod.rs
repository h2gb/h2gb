use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::H2Type;
use crate::datatype::composite::h2simple::H2Simple;
use crate::datatype::helpers::H2Context;

pub mod h2integer;
pub mod h2pointer;

use h2integer::H2Integer;
use h2pointer::H2Pointer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2BasicType {
    Integer(H2Integer),
    Pointer(H2Pointer),
}

impl From<H2BasicType> for H2Type {
    fn from(o: H2BasicType) -> H2Type {
        H2Type::from(H2Simple::new(o))
    }
}

impl H2BasicType {
    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        match self {
            Self::Integer(t) => t.to_string(context),
            Self::Pointer(t) => t.to_string(context),
        }
    }

    pub fn related(&self, context: &H2Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        match self {
            Self::Integer(t) => t.related(context),
            Self::Pointer(t) => t.related(context),
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            Self::Integer(t) => t.size(),
            Self::Pointer(t) => t.size(),
        }
    }
}
