use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::helpers::h2context::{H2Context, NumberDefinition, NumberSize};
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

    pub fn name(&self) -> String {
        match self.format.size() {
            NumberSize::Eight     => "db",
            NumberSize::Sixteen   => "dw",
            NumberSize::ThirtyTwo => "dd",
            NumberSize::SixtyFour => "dq",
        }.to_string()
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        context.read_number_as_string(self.format)
    }

    pub fn length(&self) -> usize {
        self.format.size().len()
    }

    pub fn related(&self, _context: &H2Context) -> SimpleResult<Vec<(usize, H2Type)>> {
        Ok(vec![])
    }
}
