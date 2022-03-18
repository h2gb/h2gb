use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};
use std::ops::Range;

use crate::project::H2Project;
use crate::project::H2Buffer;
use crate::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    name: String,
    source_buffer: String,
    range: Range<usize>,
    base_address: usize,
}

// TODO(ron) It'd be nice to store this with the buffer so we can get it without storing it here
#[derive(Serialize, Deserialize, Debug)]
struct Backward {
    name: String,
    source_buffer: String,
    range: Range<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferExtract(State);

impl ActionBufferExtract {
    pub fn new(name: impl AsRef<str>, source_buffer: impl AsRef<str>, range: Range<usize>, base_address: usize) -> Action {
        Action::BufferExtract(
            ActionBufferExtract(
                State::Forward(Forward {
                    name: name.as_ref().to_string(),
                    source_buffer: source_buffer.as_ref().to_string(),
                    range: range.clone(),
                    base_address: base_address,
                })
            )
        )
    }
}

impl Command for ActionBufferExtract {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        let source = project.buffer_get(&forward.source_buffer).ok_or(SimpleError::new("Couldn't find buffer"))?;
        let data = source.byte_range(forward.range.clone())?;
        let buffer = H2Buffer::new(&forward.name, data.to_vec(), forward.base_address)?;
        project.buffer_insert(&forward.name, buffer)?;

        // Save the backward struct
        self.0 = State::Backward(Backward {
            name: forward.name.to_string(),
            source_buffer: forward.source_buffer.to_string(),
            range: forward.range.clone(),
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the backward struct
        let backward = match &self.0 {
            State::Backward(b) => b,
            _                    => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        let name = &backward.name;
        let buffer = project.buffer_remove(name)?;

        // Save the forward struct
        self.0 = State::Forward(Forward {
            name: name.clone(),
            source_buffer: backward.source_buffer.clone(),
            range: backward.range.clone(),
            base_address: buffer.base_address,
        });

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use simple_error::SimpleResult;

//     use crate::project::H2Project;
//     use redo::Record;
//     use pretty_assertions::assert_eq;

//     #[test]
//     fn test_action() -> SimpleResult<()> {
//         Ok(())
//     }
// }
