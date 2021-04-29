use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;
use crate::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    buffer: String,
    layer: String,
    offset: usize,
    comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Backward {
    buffer: String,
    layer: String,
    offset: usize,
    old_comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionEntrySetComment(State);

impl ActionEntrySetComment {
    pub fn new(buffer: &str, layer: &str, offset: usize, comment: Option<String>) -> Action {
        Action::EntrySetComment(
            ActionEntrySetComment(
                State::Forward(Forward {
                    buffer: buffer.to_string(),
                    layer: layer.to_string(),
                    offset: offset,
                    comment: comment,
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

        let old_comment = project.comment_set(&forward.buffer, &forward.layer, forward.offset, forward.comment.clone())?;

        // Save the backward struct
        // Gotta save enough to know where to find it
        self.0 = State::Backward(Backward {
            buffer: forward.buffer.clone(),
            layer: forward.layer.clone(),
            offset: forward.offset,
            old_comment: old_comment,
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
        let original_comment = project.comment_set(&backward.buffer, &backward.layer, backward.offset, backward.old_comment.clone())?;

        // Save the backward struct
        self.0 = State::Forward(Forward {
            buffer: backward.buffer.clone(),
            layer: backward.layer.clone(),
            offset: backward.offset,
            comment: original_comment,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
}
