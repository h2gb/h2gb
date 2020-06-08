use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::h2buffer::H2Buffer;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Project {
    name: String,
    version: String,
    buffers: HashMap<String, H2Buffer>,
}
