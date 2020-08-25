use std::fmt;

use multi_vector::MultiVector;
use serde::{Serialize, Deserialize};

// H2Project is the very core, and the root of undo. All actions will be taken
// via this object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Project {
    pub name: String,
    pub version: String,
    pub revision: usize,

    buffers: MultiVector<()>,
}

impl H2Project {
    pub fn new(name: &str, version: &str) -> Self {
        H2Project {
            name: String::from(name),
            version: String::from(version),
            revision: 0,

            buffers: MultiVector::new(),
        }
    }
}

impl fmt::Display for H2Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}, version: {}, revision: {}", self.name, self.version, self.revision)
    }
}
