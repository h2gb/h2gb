use serde::{Serialize, Deserialize};

use crate::datatype::H2Type;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    // An array of strings and types (which might be other types)
    fields: Vec<(String, H2Type)>,
    byte_alignment: Option<usize>,
}
