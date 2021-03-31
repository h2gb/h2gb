//! Delete a buffer.
//!
//! In order to keep data consistent, this will only delete a buffer if it is
//! unpopulated (no layers or entries). Those will need to be deleted before the
//! buffer itself can be deleted.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;
use crate::h2buffer::H2Buffer;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferDeleteForward {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferDeleteBackward {
    name: String,
    buffer: H2Buffer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferDelete {
    forward: Option<ActionBufferDeleteForward>,
    backward: Option<ActionBufferDeleteBackward>,
}

impl ActionBufferDelete {
    pub fn new(forward: ActionBufferDeleteForward) -> Self {
        ActionBufferDelete {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<&str> for ActionBufferDelete {
    fn from(o: &str) -> Self {
        ActionBufferDelete {
            forward: Some(ActionBufferDeleteForward {
                name: o.to_string()
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferDelete {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let forward = match &self.forward {
            Some(b) => b,
            None => bail!("Failed to apply: missing context"),
        };

        let name = &forward.name;
        let buffer = project.buffer_remove(name)?;

        self.backward = Some(ActionBufferDeleteBackward {
            name: name.clone(),
            buffer: buffer,
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(f) => f,
            None => bail!("Failed to undo: missing context"),
        };

        // I don't love cloning here, but it's required to keep the object in
        // a consistent state if there's an error in buffer_insert()
        project.buffer_insert(&backward.name, backward.buffer.clone_shallow(None)?)?;

        self.forward = Some(ActionBufferDeleteForward {
            name: backward.name.clone(),
        });
        self.backward = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use redo::Record;
    use simple_error::SimpleResult;

    use crate::transformation::H2Transformation;

    use crate::h2project::H2Project;
    use crate::action::Action;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Start with none
        assert_eq!(0, record.target().buffers().len());

        // Create one
        record.apply(Action::buffer_create_empty("buffer", 10, 0x80000000))?;
        assert_eq!(1, record.target().buffers().len());

        // Create another
        record.apply(Action::buffer_create_empty("buffer2", 10, 0x80000000))?;
        assert_eq!(2, record.target().buffers().len());

        // Delete one
        record.apply(Action::buffer_delete("buffer2"))?;
        assert_eq!(1, record.target().buffers().len());

        // Fail to delete one that doesn't exist
        assert!(record.apply(Action::buffer_delete("buffer_fake")).is_err());
        assert_eq!(1, record.target().buffers().len());

        // Delete the other
        record.apply(Action::buffer_delete("buffer"))?;
        assert_eq!(0, record.target().buffers().len());

        // Undo the two deletes
        record.undo()?;
        record.undo()?;
        assert_eq!(2, record.target().buffers().len());

        // Redo them
        record.redo()?;
        record.redo()?;
        assert_eq!(0, record.target().buffers().len());

        Ok(())
    }

    #[test]
    fn test_action_keeps_transformations() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a simple buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"4a4B4c4D4e".to_vec(), 0x80000000))?;

        // Transform it
        record.apply(Action::buffer_transform("buffer", H2Transformation::FromHex))?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        // Delete
        record.apply(Action::buffer_delete("buffer"))?;

        // Undo the delete
        record.undo()?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        // Make sure it can still untransform
        record.apply(Action::buffer_untransform("buffer"))?;
        assert_eq!(b"4a4b4c4d4e".to_vec(), record.target().get_buffer("buffer")?.data);

        Ok(())
    }

    #[test]
    fn test_action_fails_when_buffer_is_populated() -> SimpleResult<()> {
        Ok(())
    }
}
