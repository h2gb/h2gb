use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::datatype::H2Type;
use crate::project::h2project::H2Project;
use crate::project::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    buffer: String,
    layer: String,
    datatype: H2Type,
    offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Backward {
    buffer: String,
    layer: String,
    offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionEntrySetComment(State);

impl ActionEntrySetComment {
    pub fn new(buffer: &str, layer: &str, datatype: H2Type, offset: usize) -> Action {
        Action::EntrySetComment(
            ActionEntrySetComment(
                State::Forward(Forward {
                    buffer: buffer.to_string(),
                    layer: layer.to_string(),
                    datatype: datatype,
                    offset: offset,
                })
            )
        )
    }
}

impl Command for ActionEntrySetComment {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Create the entry and saved the ResolvedType
        project.entry_create_from_type(&forward.buffer, &forward.layer, forward.datatype.clone(), forward.offset)?;

        // Save the backward struct
        // Gotta save enough to know where to find it
        self.0 = State::Backward(Backward {
            buffer: forward.buffer.clone(),
            layer: forward.layer.clone(),
            offset: forward.offset,
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let backward = match &self.0 {
            State::Backward(f) => f,
            _                  => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        // Remove the entry
        let mut entry = project.entry_remove(&backward.buffer, &backward.layer, backward.offset)?;

        // Grab the fields from it (hopefully it has an entry!)
        let (buffer, layer, datatype, offset) = match entry.pop() {
            Some(e) => e,
            None => bail!("Something went wrong with undoing the created entry! Removing the entry removed {} entries instead of just one", entry.len()),
        };

        // Sanity check - should never happen
        if offset != backward.offset {
            bail!("Something went wrong with undoing the created entry! The offset should have been {} but was actually {}", backward.offset, offset);
        }

        // This would also be really bad
        let datatype = match datatype {
            Some(d) => d,
            None => bail!("Something went wrong with undoing the created entry! The entry didn't have an H2Type associated with it"),
        };

        // Save the backward struct
        self.0 = State::Forward(Forward{
            buffer: buffer,
            layer: layer,
            datatype: datatype,
            offset: offset,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
}
