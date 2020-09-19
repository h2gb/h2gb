use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;

use h2transformer::H2Transformation;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferUntransformForward {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferUntransformBackward {
    name: String,
    original_data: Vec<u8>,
    transformation: H2Transformation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferUntransform {
    forward: Option<ActionBufferUntransformForward>,
    backward: Option<ActionBufferUntransformBackward>,
}

impl ActionBufferUntransform {
    pub fn new(forward: ActionBufferUntransformForward) -> Self {
        ActionBufferUntransform {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<&str> for ActionBufferUntransform {
    fn from(o: &str) -> Self {
        ActionBufferUntransform {
            forward: Some(ActionBufferUntransformForward {
                name: o.to_string(),
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferUntransform {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match self.forward.take() {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Apply the change
        let (original_data, transformation) = project.buffer_untransform(&forward.name)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferUntransformBackward {
            name: forward.name,
            original_data: original_data,
            transformation: transformation,
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match self.backward.take() {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        let name = backward.name;
        project.buffer_untransform_undo(&name, backward.original_data, backward.transformation)?;

        self.forward = Some(ActionBufferUntransformForward {
            name: name,
        });

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
    fn test_buffer_untransform() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with "JKLMN" encoded as hex then base64 - I alternate
        // case to ensure it normalizes when it should
        // $ echo -ne '4a4B4c4D4e' | base64 NGE0QjRjNEQ0ZQ==
        record.apply(Action::buffer_create_from_bytes("buffer", b"NGE0QjRjNEQ0ZQ==".to_vec(), 0x80000000))?;

        // Do a couple transformations, verify they worked right
        record.apply(Action::buffer_transform("buffer", H2Transformation::FromBase64))?;
        record.apply(Action::buffer_transform("buffer", H2Transformation::FromHex))?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        // Untransform one layer, which will encode back to hex - note that the
        // case is normalized
        record.apply(Action::buffer_untransform("buffer"))?;
        assert_eq!(b"4a4b4c4d4e".to_vec(), record.target().get_buffer("buffer")?.data);

        // Untransform another layer - this won't quite match the original due
        // to the case changing
        record.apply(Action::buffer_untransform("buffer"))?;
        assert_eq!(b"NGE0YjRjNGQ0ZQ==".to_vec(), record.target().get_buffer("buffer")?.data);

        // Undo the first layer
        record.undo()?;
        assert_eq!(b"4a4b4c4d4e".to_vec(), record.target().get_buffer("buffer")?.data);

        // Undo the second
        record.undo()?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        // Make sure the "transform" undos are still good
        record.undo()?;
        record.undo()?;
        assert_eq!(b"NGE0QjRjNEQ0ZQ==".to_vec(), record.target().get_buffer("buffer")?.data);
        record.redo()?;
        record.redo()?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        // Redo the untransforms
        record.redo()?;
        assert_eq!(b"4a4b4c4d4e".to_vec(), record.target().get_buffer("buffer")?.data);

        record.redo()?;
        assert_eq!(b"NGE0YjRjNGQ0ZQ==".to_vec(), record.target().get_buffer("buffer")?.data);

        println!("{}", serde_json::to_string_pretty(&record).unwrap());

        Ok(())
    }

    // #[test]
    // fn test_buffer_impossible_transform() -> SimpleResult<()> {
    //     let mut record: Record<Action> = Record::new(
    //         H2Project::new("name", "1.0")
    //     );

    //     // Definitely not hex
    //     record.apply(Action::buffer_create_from_bytes("buffer", b"abcxyz".to_vec(), 0x80000000))?;

    //     // Try to unhex
    //     assert!(record.apply(Action::buffer_transform("buffer", H2Transformation::FromHex)).is_err());

    //     // Make sure nothing changed
    //     assert_eq!(b"abcxyz".to_vec(), record.target().get_buffer("buffer")?.data);

    //     Ok(())
    // }

    // #[test]
    // fn test_fails_when_buffer_is_populated() -> SimpleResult<()> {
    //     // TODO: Fill in when I can create layers
    //     Ok(())
    // }
}
