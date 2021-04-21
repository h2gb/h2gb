//! A data buffer, where the actual bytes are stored in an h2gb project.
//!
//! When data is created or imported by h2gb, the actual raw bytes end up here.
//! They can be transformed and edited (with restrictions).
//!
//! Conceptionally below the buffer is the context of a "layer", which is where
//! data is annotated and analyzed.
//!
//! All actions done on this buffer are designed to be reversible - anything
//! that changes is returned so it can be restored later, if desired. That's
//! a very important design principle!
//!
//! In general, you should never have mutable access to a buffer or project.
//! All changes should be made via actions, otherwise the undo/redo logic will
//! fall apart!
//!
//! In other words: DON'T USE THESE DIRECTLY, unless you're writing actions.

// use std::mem;

use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
// use std::collections::HashMap;
// use std::ops::Range;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Layer {
    name: String,
}

impl H2Layer {
    pub fn new(name: &str) -> SimpleResult<Self> {
        Ok(H2Layer {
            name: String::from(name),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
}
