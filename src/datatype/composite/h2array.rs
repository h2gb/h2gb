use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};
use std::iter;

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
        let mut result = Vec::new();

        for _ in iter::repeat(self.length) {
            result.append(&mut self.field_type.to_simple_types());
        }

        result
    }

    pub fn to_string(&self, _context: &H2Context) -> SimpleResult<String> {
        // let t = self.to_simple_types()
        Ok("array".to_string())
    }
}
