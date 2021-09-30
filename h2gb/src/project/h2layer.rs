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

use std::ops::Range;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail, SimpleError};

use bumpy_vector::BumpyVector;
use h2datatype::{H2Type, ResolvedType};
use crate::project::H2Entry;

/// Hold information for a layer - basically, a bunch of entires in a
/// [`bumpy_vector::BumpyVector`].
///
/// This is actually mostly a thin wrapper around [`bumpy_vector::BumpyVector`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Layer {
    name: String,

    entries: BumpyVector<H2Entry>,
    comments: HashMap<usize, String>,
}

// impl fmt::Display for H2Layer {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(f, " Layer: {}", self.name)?;

//         for entry in self.entries.into_iter() {
//             writeln!(f, "  {}", entry.entry)?;
//         }

//         Ok(())
//     }
// }

impl H2Layer {
    pub fn new(name: &str, size: usize) -> Self {
        H2Layer {
            name: name.to_string(),
            entries: BumpyVector::new(size),
            comments: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entry_create(&mut self, resolved_type: ResolvedType, origin: Option<H2Type>) -> SimpleResult<()> {
        self.entries.insert_auto(H2Entry::new(resolved_type, origin))
    }

    pub fn entry_remove(&mut self, offset: usize) -> SimpleResult<Option<(ResolvedType, Option<H2Type>)>> {
        if offset >= self.entries.max_size() {
            bail!("Tried to remove entry at illegal offset {}", offset);
        }

        Ok(self.entries.remove(offset).map(|entry| {
            entry.entry.split_up()
        }))
    }

    pub fn entry_remove_range(&mut self, range: Range<usize>) -> SimpleResult<Vec<(ResolvedType, Option<H2Type>)>> {
        if range.is_empty() || range.end >= self.entries.max_size() {
            bail!("Tried to remove entries at illegal range {:?}", range);
        }

        Ok(self.entries.remove_range(range).into_iter().map(|entry| entry.entry.split_up()).collect())
    }

    pub fn entry_get(&self, offset: usize) -> SimpleResult<Option<H2Entry>> {
        if offset >= self.entries.max_size() {
            bail!("Tried to get entry at illegal offset {}", offset);
        }

        Ok(self.entries.get(offset).map(|entry| entry.entry.clone()))
    }

    pub fn entry_get_or_err(&self, offset: usize) -> SimpleResult<H2Entry> {
        self.entry_get(offset)?.ok_or(
            SimpleError::new(format!("No entry at offset {}", offset))
        )
    }

    pub fn entry_get_mut(&mut self, offset: usize) -> SimpleResult<Option<&mut H2Entry>> {
        if offset >= self.entries.max_size() {
            bail!("Tried to get entry at illegal offset {}", offset);
        }

        Ok(self.entries.get_mut(offset).map(|entry| &mut entry.entry))
    }

    pub fn entry_get_mut_or_err(&mut self, offset: usize) -> SimpleResult<&mut H2Entry> {
        self.entry_get_mut(offset)?.ok_or(
            SimpleError::new(format!("No entry at offset {}", offset))
        )
    }

    pub fn entries_get(&self, range: Range<usize>) -> SimpleResult<Vec<&H2Entry>> {
        if range.is_empty() || range.end >= self.entries.max_size() {
            bail!("Tried to get entries at illegal range {:?}", range);
        }

        Ok(self.entries.get_range(range).into_iter().map(|entry| &entry.entry).collect())
    }

    // pub fn entries(&self) -> &BumpyVector<H2Entry> {
    //     &self.entries
    // }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_populated(&self) -> bool {
        self.len() > 0
    }

    pub fn comment_get(&self, offset: usize) -> SimpleResult<Option<&String>> {
        if offset >= self.entries.max_size() {
            bail!("Tried to put comment at illegal offset {}", offset);
        }

        Ok(self.comments.get(&offset))
    }

    pub fn comments_get(&self, range: Range<usize>) -> SimpleResult<Vec<&String>> {
        if range.end > self.entries.max_size() {
            bail!("Tried to get comment at illegal range 0x{:x?} (max = 0x{:x?})", range, self.entries.max_size());
        }

        let mut out = Vec::new();

        for offset in range {
            match self.comment_get(offset)? {
                Some(comment) => out.push(comment),
                None => (),
            }
        }

        Ok(out)
    }

    pub fn comment_set(&mut self, offset: usize, comment: Option<String>) -> SimpleResult<Option<String>> {
        if offset >= self.entries.max_size() {
            bail!("Tried to put comment at illegal offset {}", offset);
        }

        match comment {
            Some(comment) => Ok(self.comments.insert(offset, comment)),
            None => Ok(self.comments.remove(&offset)),
        }
    }
}
