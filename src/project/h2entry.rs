//! Implement the core project, which everything else builds off of.
//!
//! This is likely to change a lot as I get the foundations built, so it's not
//! quite ready for detailed comments just yet. :)

use serde::{Serialize, Deserialize};
//use simple_error::{bail, SimpleResult};
use std::fmt;
use std::ops::Range;

//use crate::project::h2datatype::H2Type;

use crate::bumpy_vector::AutoBumpyEntry;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Entry {
    pub range: Range<usize>,
    pub display: String,
    // pub creator: Option<()>,
    // pub creations: Option<Vec<()>>,
    // pub references: Option<Vec<()>>,
    //pub datatype: H2Type,
    //pub transformations: Transformation
}

impl fmt::Display for H2Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}: {}\n", self.range.start, self.range.end, self.display)
    }
}

impl AutoBumpyEntry for H2Entry {
    fn range(&self) -> Range<usize> { self.range.clone() }
}

impl H2Entry {
    fn new(range: Range<usize>, display: String) -> Self {
        Self {
            range: range.clone(),
            display: display,
        }
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