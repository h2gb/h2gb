use serde::{Serialize, Deserialize};
use crate::datatype::H2TypeTrait;

pub mod ntstring;
use ntstring::NTString;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2DynamicType {
    NTString(NTString),
}

impl H2DynamicType {
    pub fn as_trait(&self) -> Box<&dyn H2TypeTrait> {
        match self {
            H2DynamicType::NTString(t)  => Box::new(t),
        }
    }
}
