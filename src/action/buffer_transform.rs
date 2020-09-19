use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;

use h2transformer::H2Transformation;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferTransformForward {
    pub name: String,
    pub transformation: H2Transformation,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferTransformBackward {
    name: String,
    original_data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferTransform {
    forward: Option<ActionBufferTransformForward>,
    backward: Option<ActionBufferTransformBackward>,
}

impl ActionBufferTransform {
    pub fn new(forward: ActionBufferTransformForward) -> Self {
        ActionBufferTransform {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, H2Transformation)> for ActionBufferTransform {
    fn from(o: (&str, H2Transformation)) -> Self {
        ActionBufferTransform {
            forward: Some(ActionBufferTransformForward {
                name: o.0.to_string(),
                transformation: o.1,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferTransform {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Apply the change
        let original_data = project.buffer_transform(&forward.name, forward.transformation)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferTransformBackward {
            name: forward.name.clone(),
            original_data: original_data,
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        // Cloning the original_data is necessary for keeping the record
        // consistent if there's an error
        let transformation = project.buffer_transform_undo(&backward.name, backward.original_data.clone())?;

        self.forward = Some(ActionBufferTransformForward {
            name: backward.name.clone(),
            transformation: transformation,
        });
        self.backward = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use simple_error::SimpleResult;

    use crate::h2project::H2Project;
    use redo::Record;
    use pretty_assertions::assert_eq;
    use crate::action::Action;
    use h2transformer::H2Transformation;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with "JKLMN" encoded as hex then base64 - I alternate
        // case to ensure it's undoing correctly:
        // $ echo -ne '4a4B4c4D4e' | base64 NGE0QjRjNEQ0ZQ==
        record.apply(Action::buffer_create_from_bytes("buffer", b"NGE0QjRjNEQ0ZQ==".to_vec(), 0x80000000))?;
        assert_eq!(b"NGE0QjRjNEQ0ZQ==".to_vec(), record.target().get_buffer("buffer")?.data);

        // Undo the base64
        record.apply(Action::buffer_transform("buffer", H2Transformation::FromBase64))?;
        assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().get_buffer("buffer")?.data);

        record.apply(Action::buffer_transform("buffer", H2Transformation::FromHex))?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        // Undo both
        record.undo()?;
        assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().get_buffer("buffer")?.data);

        record.undo()?;
        assert_eq!(b"NGE0QjRjNEQ0ZQ==".to_vec(), record.target().get_buffer("buffer")?.data);

        // Redo them
        record.redo()?;
        assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().get_buffer("buffer")?.data);

        record.redo()?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        Ok(())
    }

    #[test]
    fn test_action_fails_on_impossible_transform() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Definitely not hex
        record.apply(Action::buffer_create_from_bytes("buffer", b"abcxyz".to_vec(), 0x80000000))?;

        // Try to unhex
        assert!(record.apply(Action::buffer_transform("buffer", H2Transformation::FromHex)).is_err());

        // Make sure nothing changed
        assert_eq!(b"abcxyz".to_vec(), record.target().get_buffer("buffer")?.data);

        Ok(())
    }

    #[test]
    fn test_action_fails_when_buffer_is_populated() -> SimpleResult<()> {
        // TODO: Fill in when I can create layers
        Ok(())
    }
}
