use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::h2transformation::H2Transformation;
use crate::h2layer::H2Layer;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Buffer {
    // Name
    name: String,

    // Base address
    base_address: usize,

    // Memory block
    data: Vec<u8>,

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

impl H2Buffer {
    // fn from_bytes(name: &str, base_address: usize, data: Vec<u8>) -> Self {
    // }

    // fn from_h2buffer(name: &str, base_address: usize, data: &H2Buffer, start: usize, length: usize) -> Self {
    // }

    // fn transform(transformer: &H2Transformation) -> Result<(), &'static str> {
    //     Ok(())
    // }
}
