use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;
use crate::project::actions::Action;
use crate::transformation::Transformation;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    name: String,
    transformation: Transformation,
}

#[derive(Serialize, Deserialize, Debug)]
struct Backward {
    name: String,
    original_data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferTransform(State);

impl ActionBufferTransform {
    pub fn new(name: &str, transformation: Transformation) -> Action {
        Action::BufferTransform(
            ActionBufferTransform(
                State::Forward(Forward {
                    name: String::from(name),
                    transformation: transformation,
                })
            )
        )
    }
}

impl Command for ActionBufferTransform {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Get a handle to the buffer
        let buffer = match project.buffer_get_mut(&forward.name) {
            Some(b) => b,
            None => bail!("Could not find buffer {} to transform", &forward.name),
        };

        // Transform the buffer, and get the original data (for undo)
        let original_data = buffer.transform(forward.transformation)?;

        // Save the backward struct
        self.0 = State::Backward(Backward {
            name: forward.name.to_string(),
            original_data: original_data,
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the backward struct
        let backward = match &self.0 {
            State::Backward(b) => b,
            _                    => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        // Get a handle to the buffer
        let buffer = match project.buffer_get_mut(&backward.name) {
            Some(b) => b,
            None => bail!("Could not find buffer {} to under the transformation", &backward.name),
        };

        // Do the undo and save the transformation
        let transformation = buffer.transform_undo(backward.original_data.clone())?;

        // Save the forward struct
        self.0 = State::Forward(Forward {
            name: backward.name.clone(),
            transformation: transformation,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use redo::Record;
    use pretty_assertions::assert_eq;

    use crate::transformation::{TransformHex, TransformBase64};
    use crate::project::actions::{Action, ActionBufferCreateFromBytes};

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with "JKLMN" encoded as hex then base64 - I alternate
        // case to ensure it's undoing correctly:
        // $ echo -ne '4a4B4c4D4e' | base64 NGE0QjRjNEQ0ZQ==
        let action = ActionBufferCreateFromBytes::new("buffer", &b"NGE0QjRjNEQ0ZQ==".to_vec(), 0x80000000);
        record.apply(action)?;
        assert_eq!(b"NGE0QjRjNEQ0ZQ==".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        // Undo the base64
        let action = ActionBufferTransform::new("buffer", TransformBase64::standard());
        record.apply(action)?;
        assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        let action = ActionBufferTransform::new("buffer", TransformHex::new());
        record.apply(action)?;
        assert_eq!(b"JKLMN".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        // Undo both
        record.undo()?;
        assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        record.undo()?;
        assert_eq!(b"NGE0QjRjNEQ0ZQ==".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        // Redo them
        record.redo()?;
        assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        record.redo()?;
        assert_eq!(b"JKLMN".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        Ok(())
    }

    #[test]
    fn test_action_fails_on_impossible_transform() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Definitely not hex
        let action = ActionBufferCreateFromBytes::new("buffer", &b"abcxyz".to_vec(), 0x80000000);
        record.apply(action)?;

        // Try to unhex
        let action = ActionBufferTransform::new("buffer", TransformHex::new());
        assert!(record.apply(action).is_err());

        // Make sure nothing changed
        assert_eq!(b"abcxyz".to_vec(), record.target().buffer_get("buffer").unwrap().data);

        Ok(())
    }

    // #[test]
    // fn test_action_fails_when_buffer_is_populated() -> SimpleResult<()> {
    //     Ok(())
    // }
}
