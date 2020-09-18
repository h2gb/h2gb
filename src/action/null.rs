use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError};

use crate::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct NullAction {
}

impl NullAction {
    pub fn new() -> Self {
        NullAction {
        }
    }
}

impl From<()> for NullAction {
    fn from(_o: ()) -> Self {
        NullAction {}
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
