use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::H2Type;
use crate::datatype::basic::H2BasicType;
use crate::datatype::helpers::sized_number::{SizedFormat, SizedDisplay};
use crate::datatype::helpers::H2Context;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Integer {
    number_format: SizedFormat,
    display_format: SizedDisplay,
}

impl From<H2Integer> for H2Type {
    fn from(o: H2Integer) -> H2Type {
        H2Type::from(H2BasicType::Integer(o))
    }
}

impl H2Integer {
    pub fn new(number_format: SizedFormat, display_format: SizedDisplay) -> Self {
        Self {
            number_format: number_format,
            display_format: display_format,
        }
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        let number = self.number_format.read(context)?;
        Ok(self.display_format.to_string(number))
    }

    pub fn size(&self) -> u64 {
        self.number_format.size()
    }

    pub fn related(&self, _context: &H2Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }
}
