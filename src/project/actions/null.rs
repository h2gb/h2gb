//! Perform no action.
//!
//! This is just meant as a test, it probably doesn't need to be implemented
//! anywhere.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError};

use crate::project::h2project::H2Project;
use crate::project::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
pub struct NullAction {
}

impl NullAction {
    pub fn new() -> Action {
        Action::Null(NullAction {
        })
    }
}

impl Command for NullAction {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, _t: &mut H2Project) -> SimpleResult<()> {
        Ok(())
    }

    fn undo(&mut self, _t: &mut H2Project) -> SimpleResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::{SimpleResult, bail};
    use pretty_assertions::assert_eq;
    use redo::Record;

    #[test]
    fn test_null() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        let action = NullAction::new();

        record.apply(action)?;

        Ok(())
    }
}
