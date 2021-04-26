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

use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::ops::Range;

use crate::bumpy_vector::BumpyVector;
use crate::project::h2entry::H2Entry;

/// Hold information for a layer - basically, a bunch of entires in a
/// [`BumpyVector`].
///
/// This is actually mostly a thin wrapper around [`BumpyVector`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Layer {
    name: String,

    entries: BumpyVector<H2Entry>,
}

impl H2Layer {
    pub fn new(name: &str, size: usize) -> Self {
        H2Layer {
            name: name.to_string(),
            entries: BumpyVector::new(size),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn insert_entry(&mut self, entry: H2Entry) -> SimpleResult<()> {
        self.entries.insert_auto(entry)
    }

    pub fn remove_entry(&mut self, offset: usize) -> Option<H2Entry> {
        self.entries.remove(offset).map(|entry| {
            entry.entry
        })
    }

    pub fn remove_range(&mut self, range: Range<usize>) -> Vec<H2Entry> {
        self.entries.remove_range(range).into_iter().map(|entry| entry.entry).collect()
    }

    pub fn get_entry(&mut self, index: usize) -> Option<&H2Entry> {
        self.entries.get(index).map(|entry| &entry.entry)
    }

    pub fn get_entry_mut(&mut self, index: usize) -> Option<&mut H2Entry> {
        self.entries.get_mut(index).map(|entry| &mut entry.entry)
    }

    pub fn get_entries(&self, range: Range<usize>) -> Vec<&H2Entry> {
        self.entries.get_range(range).into_iter().map(|entry| &entry.entry).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_populated(&self) -> bool {
        self.len() > 0
    }
}
