//! Implement the core project, which everything else builds off of.
//!
//! This is likely to change a lot as I get the foundations built, so it's not
//! quite ready for detailed comments just yet. :)

use serde::{Serialize, Deserialize};
use std::fmt;
use std::ops::Range;

use bumpy_vector::AutoBumpyEntry;
use h2datatype::{H2Type, ResolvedType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Entry {
    resolved_type: ResolvedType,
    origin: Option<H2Type>,
}

impl fmt::Display for H2Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.resolved_type)
    }
}

impl AutoBumpyEntry for H2Entry {
    fn range(&self) -> Range<usize> {
        // TODO: Converting like this is bad news
        (self.resolved_type.aligned_range.start as usize)..(self.resolved_type.aligned_range.end as usize)
    }
}

impl H2Entry {
    pub fn new(resolved_type: ResolvedType, origin: Option<H2Type>) -> Self {
        Self {
            resolved_type: resolved_type,
            origin: origin,
        }
    }

    pub fn resolved(&self) -> &ResolvedType {
        &self.resolved_type
    }

    pub fn origin(&self) -> &Option<H2Type> {
        &self.origin
    }

    pub fn split_up(self) -> (ResolvedType, Option<H2Type>) {
        (self.resolved_type, self.origin)
    }
}

#[cfg(test)]
mod tests {
}
