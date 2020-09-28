use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::helpers::h2context::H2Context;
use crate::datatype::H2Type;

pub mod h2integer;
pub mod h2pointer;

use h2integer::H2Integer;
use h2pointer::H2Pointer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2SimpleType {
    Integer(H2Integer),
    Pointer(H2Pointer),
}

impl H2SimpleType {
    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        match self {
            Self::Integer(t) => t.to_string(context),
            Self::Pointer(t) => t.to_string(context),
        }
    }

    pub fn related(&self, context: &H2Context) -> Vec<(usize, H2Type)> {
        match self {
            Self::Integer(t) => t.related(context),
            Self::Pointer(t) => t.related(context),
        }
    }

    pub fn length(&self) -> usize {
        match self {
            Self::Integer(t) => t.length(),
            Self::Pointer(t) => t.length(),
        }
    }
}
