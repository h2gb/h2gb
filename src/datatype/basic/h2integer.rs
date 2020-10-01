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
    pub const U8:         Self = Self { format: NumberDefinition::U8         };
    pub const U16_BIG:    Self = Self { format: NumberDefinition::U16_BIG    };
    pub const U16_LITTLE: Self = Self { format: NumberDefinition::U16_LITTLE };
    pub const U32_BIG:    Self = Self { format: NumberDefinition::U32_BIG    };
    pub const U32_LITTLE: Self = Self { format: NumberDefinition::U32_LITTLE };
    pub const U64_BIG:    Self = Self { format: NumberDefinition::U64_BIG    };
    pub const U64_LITTLE: Self = Self { format: NumberDefinition::U64_LITTLE };

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
