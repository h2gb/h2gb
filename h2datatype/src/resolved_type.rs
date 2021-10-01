use serde::{Serialize, Deserialize};

use std::fmt;
use std::ops::Range;

use crate::H2Type;
use generic_number::{GenericNumber, Integer, Float, Character};

/// The result of an [`H2Type`] being applied to a context.
///
/// Ultimately, a [`ResolvedType`] is "concrete" - that is, everything is
/// set in stone and unchanging. That means that getting information from it
/// is zero-cost, and requests can't fail (since they aren't working on
/// unexpected data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedType {
    pub actual_range: Range<u64>,
    pub aligned_range: Range<u64>,

    pub field_name: Option<String>,
    pub display: String,

    pub children: Vec<ResolvedType>,
    pub related: Vec<(u64, H2Type)>,

    pub as_string: Option<String>,
    pub as_number: Option<GenericNumber>, // XXX Deprecated

    pub as_integer:   Option<Integer>,
    pub as_float:     Option<Float>,
    pub as_character: Option<Character>,
}

impl ResolvedType {
    pub fn actual_size(&self) -> u64 {
        self.actual_range.end - self.actual_range.start
    }

    pub fn aligned_size(&self) -> u64 {
        self.aligned_range.end - self.aligned_range.start
    }
}

impl fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display)
    }
}
