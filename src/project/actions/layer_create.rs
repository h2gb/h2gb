//! Create a new buffer of a particular size, initialized to all zeroes.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;
use crate::project::h2layer::H2Layer;
use crate::project::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    buffer: String,
    name: String,
}

// Backward == forward here
type Backward = Forward;

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionLayerCreate(State);

impl ActionLayerCreate {
    pub fn new(buffer: &str, name: &str) -> Action {
        Action::LayerCreate(
            ActionLayerCreate(
                State::Forward(Forward {
                    buffer: String::from(buffer),
                    name: String::from(name),
                })
            )
        )
    }
}

impl Command for ActionLayerCreate {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        let buffer = project.get_buffer_mut(&forward.buffer)?;
        buffer.layer_add(H2Layer::new(&forward.name)?)?;

        // Save the backward struct
        self.0 = State::Backward(Backward {
            buffer: forward.buffer.clone(),
            name: forward.name.clone(),
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
        let buffer = project.get_buffer_mut(&backward.buffer)?;
        buffer.layer_remove(&backward.name)?;

        // Save the forward struct
        self.0 = State::Forward(Forward {
            buffer: backward.buffer.clone(),
            name: backward.name.clone(),
        });


        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use pretty_assertions::assert_eq;
    use redo::Record;
    use simple_error::SimpleResult;

    use crate::project::h2project::H2Project;
    use crate::project::actions::ActionBufferCreateEmpty;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        let action = ActionBufferCreateEmpty::new("buffer", 100, 0);
        record.apply(action)?;

        // Create a layer in the buffer
        let action = ActionLayerCreate::new("buffer", "layer");
        record.apply(action)?;

        // Get the buffer and make sure the layer now exists
        let buffer = record.target().get_buffer("buffer")?;
        assert!(buffer.get_layer("layer").is_ok());

        record.undo()?;

        // Get the buffer and make sure the layer is gone
        let buffer = record.target().get_buffer("buffer")?;
        assert!(buffer.get_layer("layer").is_err());

        record.redo()?;

        // Get the buffer and make sure the layer is back again
        let buffer = record.target().get_buffer("buffer")?;
        assert!(buffer.get_layer("layer").is_ok());

        Ok(())
    }

    #[test]
    fn test_action_fails_if_layer_already_exists() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a pair of buffers
        let action = ActionBufferCreateEmpty::new("buffer1", 100, 0);
        record.apply(action)?;
        let action = ActionBufferCreateEmpty::new("buffer2", 100, 0);
        record.apply(action)?;

        // Create a layer in the first buffer
        let action = ActionLayerCreate::new("buffer1", "layer");
        record.apply(action)?;

        // Make sure it fails if we try again
        let action = ActionLayerCreate::new("buffer1", "layer");
        assert!(record.apply(action).is_err());

        // Make sure it succeeds on the other buffer
        let action = ActionLayerCreate::new("buffer2", "layer");
        assert!(record.apply(action).is_ok());

        // Undo / redo for good measure
        record.undo()?;
        record.undo()?;
        record.redo()?;
        record.redo()?;

        Ok(())
    }
}
