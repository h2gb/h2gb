use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::H2Type;
use crate::datatype::basic::H2BasicType;
use crate::datatype::helpers::h2context::{H2Context, NumberDefinition};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Integer {
    format: NumberDefinition,
}

impl From<H2Integer> for H2Type {
    fn from(o: H2Integer) -> H2Type {
        H2Type::from(H2BasicType::Integer(o))
    }
}

impl H2Integer {
    pub fn new(format: NumberDefinition) -> Self {
        Self {
            format: format,
        }
    }

    pub fn u8() -> Self {
        Self::new(NumberDefinition::u8())
    }

    pub fn u16_big() -> Self {
        Self::new(NumberDefinition::u16_big())
    }

    pub fn u16_little() -> Self {
        Self::new(NumberDefinition::u16_little())
    }

    pub fn u32_big() -> Self {
        Self::new(NumberDefinition::u32_big())
    }

    pub fn u32_little() -> Self {
        Self::new(NumberDefinition::u32_little())
    }

    pub fn u64_big() -> Self {
        Self::new(NumberDefinition::u64_big())
    }

    pub fn u64_little() -> Self {
        Self::new(NumberDefinition::u64_little())
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
