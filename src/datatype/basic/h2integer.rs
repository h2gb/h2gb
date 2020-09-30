use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::H2Type;
use crate::datatype::basic::H2BasicType;
use crate::datatype::helpers::h2context::{H2Context, NumberDefinition, NumberSize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Integer {
    format: NumberDefinition,
}

impl From<H2Integer> for H2BasicType {
    fn from(o: H2Integer) -> H2BasicType {
        H2BasicType::Integer(o)
    }
}

impl From<H2Integer> for H2Type {
    fn from(o: H2Integer) -> H2Type {
        H2Type::from(H2BasicType::from(o))
    }
}

impl H2Integer {
    pub fn new(format: NumberDefinition) -> Self {
        Self {
            format: format,
        }
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        context.read_number_as_string(self.format)
    }

    pub fn size(&self) -> usize {
        self.format.size().len()
    }

    pub fn related(&self, _context: &H2Context) -> SimpleResult<Vec<(usize, H2Type)>> {
        Ok(vec![])
    }
}
