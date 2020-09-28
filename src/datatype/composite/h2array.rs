use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::datatype::H2Type;
use crate::datatype::helpers::h2context::H2Context;
use crate::datatype::simple::H2SimpleType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: usize,
    byte_alignment: Option<usize>,
}

impl H2Array {
    pub fn to_simple_types(&self) -> Vec<H2SimpleType> {
        vec![]
    }

    pub fn to_string(&self, _context: &H2Context) -> SimpleResult<String> {
        Ok("array".to_string())
    }
}
