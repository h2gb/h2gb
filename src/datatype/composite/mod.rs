use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::simple::H2SimpleType;
use crate::datatype::helpers::h2context::H2Context;

pub mod h2struct;
pub mod h2array;

use h2struct::H2Struct;
use h2array::H2Array;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2CompositeType {
    H2Struct(H2Struct),
    H2Array(H2Array),
}

impl H2CompositeType {
    pub fn to_simple_types(&self) -> Vec<H2SimpleType> {
        match self {
            Self::H2Struct(t) => t.to_simple_types(),
            Self::H2Array(t) => t.to_simple_types(),
        }
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        match self {
            Self::H2Struct(t) => t.to_string(context),
            Self::H2Array(t) => t.to_string(context),
        }
    }
}
