// use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::h2datatype::H2DataType;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Entry {
    address: usize,
    size: usize,
    display: String,

    // TODO(ron): I don't feel great about how I'm tracking this
    // In particular, H2Combinator isn't going to work here (do I care?)
    creator: H2DataType,
}
