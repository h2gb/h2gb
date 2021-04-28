//! Implement the core project, which everything else builds off of.
//!
//! This is likely to change a lot as I get the foundations built, so it's not
//! quite ready for detailed comments just yet. :)

use serde::{Serialize, Deserialize};
use std::fmt;
use std::ops::Range;

use crate::bumpy_vector::AutoBumpyEntry;
use crate::datatype::{H2Type, ResolvedType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Entry {
    datatype: ResolvedType,
    creator: Option<H2Type>,

    // pub creations: Option<Vec<()>>,
    // pub references: Option<Vec<()>>,
    //pub datatype: H2Type,
    //pub transformations: Transformation
}

impl fmt::Display for H2Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.datatype)
    }
}

impl AutoBumpyEntry for H2Entry {
    fn range(&self) -> Range<usize> {
        // TODO: Converting like this is bad news
        (self.datatype.aligned_range.start as usize)..(self.datatype.aligned_range.end as usize)
    }
}

impl H2Entry {
    pub fn new(datatype: ResolvedType, creator: Option<H2Type>) -> Self {
        Self {
            datatype: datatype,
            creator: creator,
        }
    }

    pub fn resolved(&self) -> &ResolvedType {
        &self.datatype
    }

    pub fn creator(&self) -> Option<H2Type> {
        self.creator.clone()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use simple_error::SimpleResult;

    #[test]
    fn test_() -> SimpleResult<()> {
        Ok(())
    }
}
