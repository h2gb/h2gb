use serde::{Serialize, Deserialize};

use crate::datatype::H2Type;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: usize,
    byte_alignment: Option<usize>,
}
