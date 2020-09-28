use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::datatype::H2Type;
use crate::datatype::helpers::h2context::H2Context;
use crate::datatype::simple::H2SimpleType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    // An array of strings and types (which might be other types)
    fields: Vec<(String, H2Type)>,
    byte_alignment: Option<usize>,
}

impl H2Struct {
    pub fn to_simple_types(&self) -> Vec<H2SimpleType> {
        vec![]
    }

    pub fn to_string(&self, _context: &H2Context) -> SimpleResult<String> {
        Ok("struct".to_string())
    }
}
