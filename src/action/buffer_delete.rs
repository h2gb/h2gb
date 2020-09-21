use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;
use crate::h2project::h2buffer::H2Buffer;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferDeleteForward {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferDeleteBackward {
    name: String,
    data: Vec<u8>,
    base_address: usize,
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
            data: buffer.data,
            base_address: buffer.base_address,
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
        let buffer = H2Buffer::new(backward.data.clone(), backward.base_address)?;
        project.buffer_insert(&backward.name, buffer)?;

        self.forward = Some(ActionBufferDeleteForward {
            name: backward.name.clone(),
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

    fn test_action_fails_when_buffer_is_populated() -> SimpleResult<()> {
        // TODO: Fill in when I can create layers
        Ok(())
    }
}
