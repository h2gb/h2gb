//! Perform no action.
//!
//! This is just meant as a test, it probably doesn't need to be implemented
//! anywhere.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;
use crate::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
}

#[derive(Serialize, Deserialize, Debug)]
struct Backward {
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NullAction(State);

impl NullAction {
    pub fn new() -> Action {
        Action::Null(NullAction(State::Forward(Forward{})))
    }
}

impl Command for NullAction {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, _t: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let _forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        // ...

        // Save the backward struct
        self.0 = State::Backward(Backward{});

        Ok(())
    }

    fn undo(&mut self, _t: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let _backward = match &self.0 {
            State::Backward(f) => f,
            _                  => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        // ...

        // Save the backward struct
        self.0 = State::Forward(Forward{});

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use redo::Record;

    #[test]
    fn test_null() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        let action = NullAction::new();

        record.apply(action)?;
        record.undo()?;
        record.redo()?;

        Ok(())
    }
}
