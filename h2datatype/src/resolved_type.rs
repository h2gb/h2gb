use serde::{Serialize, Deserialize};

use std::fmt;
use std::ops::Range;

use crate::H2Type;
use generic_number::{Integer, Float, Character};

/// The result of an [`H2Type`] being applied to a context.
///
/// Ultimately, a [`ResolvedType`] is "concrete" - that is, everything is
/// set in stone and unchanging. That means that getting information from it
/// is zero-cost, and requests can't fail (since they aren't working on
/// unexpected data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedType {
    pub base_range:    Range<usize>,
    pub aligned_range: Range<usize>,

    pub field_name:    Option<String>,
    pub display:       String,

    pub children:      Vec<ResolvedType>,
    pub related:       Vec<(usize, H2Type)>,

    pub as_string:     Option<String>,
    pub as_integer:    Option<Integer>,
    pub as_float:      Option<Float>,
    pub as_character:  Option<Character>,
}

impl ResolvedType {
    pub fn base_size(&self) -> usize {
        self.base_range.end - self.base_range.start
    }

    pub fn aligned_size(&self) -> usize {
        self.aligned_range.end - self.aligned_range.start
    }
}

impl fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display)
    }
}
