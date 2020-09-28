use serde::{Serialize, Deserialize};

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
        vec![]
    }

    pub fn to_string(&self, _context: &H2Context) -> String {
        "test".into()
    }
}
