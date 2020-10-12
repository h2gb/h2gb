use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use sized_number::Context;

use crate::datatype::H2Type;
use crate::datatype::composite::h2simple::H2Simple;

pub mod h2number;
pub mod h2pointer;

use h2number::H2Number;
use h2pointer::H2Pointer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2BasicType {
    Number(H2Number),
    Pointer(H2Pointer),
}

impl From<H2BasicType> for H2Type {
    fn from(o: H2BasicType) -> H2Type {
        H2Type::from(H2Simple::new(o))
    }
}

impl H2BasicType {
    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        match self {
            Self::Number(t)  => t.to_string(context),
            Self::Pointer(t) => t.to_string(context),
        }
    }

    pub fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        match self {
            Self::Number(t)  => t.related(context),
            Self::Pointer(t) => t.related(context),
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            Self::Number(t)  => t.size(),
            Self::Pointer(t) => t.size(),
        }
    }
}
