use serde::{Serialize, Deserialize};

use crate::datatype::H2TypeTrait;

pub mod h2array;
use h2array::H2Array;

// pub mod h2enum;
// use h2enum::H2Enum;

// pub mod h2struct;
// use h2struct::H2Struct;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2ComplexType {
    H2Array(H2Array),
    // H2Enum(H2Enum),
    // H2Struct(H2Struct),
}

impl H2ComplexType {
    pub fn as_trait(&self) -> Box<&dyn H2TypeTrait> {
        match self {
            H2ComplexType::H2Array(t)  => Box::new(t),
        }
    }
}
