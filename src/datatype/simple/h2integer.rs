use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::helpers::H2Context;
use crate::datatype::helpers::number::NumberDefinition;
use crate::datatype::simple::H2SimpleType;
use crate::datatype::H2Type;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Integer {
    format: NumberDefinition,
}

impl From<H2Integer> for H2Type {
    fn from(o: H2Integer) -> H2Type {
        H2Type::from(H2SimpleType::Integer(o))
    }
}

impl H2Integer {
    pub fn new(format: NumberDefinition) -> Self {
        Self {
            format: format,
        }
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        self.format.to_string(context)
    }

    pub fn length(&self) -> usize {
        self.format.len()
    }

    pub fn related(&self, _context: &H2Context) -> Vec<(usize, H2Type)> {
        vec![]
    }
}
