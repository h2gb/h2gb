#![allow(dead_code)]
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use bumpy_vector::BumpyVector;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Action {
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Query {
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Project {
    name: String,
    version: String,
    buffers: HashMap<String, H2Buffer>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Buffer {
    // Name
    name: String,

    // Base address
    base_address: usize,

    // Memory block
    memory: Vec<u8>,

    // Child buffers (by name)
    children: Vec<String>,

    // Related buffers (by name)
    related: Vec<String>,

    // Editable (TODO: Do I actually need this?)
    editable: bool,

    // List of transformations that got us here
    transformations: Vec<H2Transformation>,

    // List of layers, indexed by name
    layers: HashMap<String, H2Layer>,

    // List of cross-references (both inside and outside)
    // TODO(ron): Not sure what type these will be yet
    cross_references: (),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Layer {
    name: String,
    show_undefined: bool,
    entries: BumpyVector<H2Entry>,
    combinators: BumpyVector<H2Combinator>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Entry {
    address: usize,
    size: usize,
    display: String,

    // TODO(ron): I don't feel great about how I'm tracking this
    // In particular, H2Combinator isn't going to work here (do I care?)
    creator: H2DataType,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2DataType {
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Combinator {
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Transformation {
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Analyzer {
}

#[cfg(test)]
mod tests {
    // use super::*;
    use bumpy_vector::BumpyVector;

    #[test]
    fn test_vector() {
        let h: BumpyVector<u32> = BumpyVector::new(1);

        let s = ron::ser::to_string(&h).unwrap();
        println!("Serialized: {}", s);
    }
}
