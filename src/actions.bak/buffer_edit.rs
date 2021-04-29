//! Replace part of a buffer's data, maintaining the original length.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferEditForward {
    pub name: String,
    pub new_data: Vec<u8>,
    pub offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferEditBackward {
    name: String,
    original_data: Vec<u8>,
    offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferEdit {
    forward: Option<ActionBufferEditForward>,
    backward: Option<ActionBufferEditBackward>,
}

impl ActionBufferEdit {
    pub fn new(forward: ActionBufferEditForward) -> Self {
        ActionBufferEdit {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, Vec<u8>, usize)> for ActionBufferEdit {
    fn from(o: (&str, Vec<u8>, usize)) -> Self {
        ActionBufferEdit {
            forward: Some(ActionBufferEditForward {
                name: o.0.to_string(),
                new_data: o.1,
                offset: o.2,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferEdit {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Apply the change
        let buffer = project.get_buffer_mut(&forward.name)?;
        let original_data = buffer.edit(forward.new_data.clone(), forward.offset)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferEditBackward {
            name: forward.name.clone(),
            original_data: original_data,
            offset: forward.offset,
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        let buffer = project.get_buffer_mut(&backward.name)?;
        let new_data = buffer.edit(backward.original_data.clone(), backward.offset)?;

        self.forward = Some(ActionBufferEditForward {
            name: backward.name.clone(),
            new_data: new_data,
            offset: backward.offset,
        });
        self.backward = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use simple_error::SimpleResult;

    use crate::project::h2project::H2Project;
    use redo::Record;
    use pretty_assertions::assert_eq;
    use crate::actions::Action;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change the middle three bytes
        record.apply(Action::buffer_edit("buffer", b"BBBBBB".to_vec(), 2))?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change the last two bytes
        record.apply(Action::buffer_edit("buffer", b"CC".to_vec(), 8))?;
        assert_eq!(b"AABBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change the first two bytes
        record.apply(Action::buffer_edit("buffer", b"EE".to_vec(), 0))?;
        assert_eq!(b"EEBBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change the first one byte
        record.apply(Action::buffer_edit("buffer", b"F".to_vec(), 0))?;
        assert_eq!(b"FEBBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change the last one byte
        record.apply(Action::buffer_edit("buffer", b"G".to_vec(), 9))?;
        assert_eq!(b"FEBBBBBBCG".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change the whole vector
        record.apply(Action::buffer_edit("buffer", b"ZZZZZZZZZZ".to_vec(), 0))?;
        assert_eq!(b"ZZZZZZZZZZ".to_vec(), record.target().get_buffer("buffer")?.data);

        // Make sure each step can undo
        record.undo()?;
        assert_eq!(b"FEBBBBBBCG".to_vec(), record.target().get_buffer("buffer")?.data);
        record.undo()?;
        assert_eq!(b"FEBBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);
        record.undo()?;
        assert_eq!(b"EEBBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);
        record.undo()?;
        assert_eq!(b"AABBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);
        record.undo()?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);
        record.undo()?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Make sure each step can redo
        record.redo()?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);
        record.redo()?;
        assert_eq!(b"AABBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);
        record.redo()?;
        assert_eq!(b"EEBBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);
        record.redo()?;
        assert_eq!(b"FEBBBBBBCC".to_vec(), record.target().get_buffer("buffer")?.data);
        record.redo()?;
        assert_eq!(b"FEBBBBBBCG".to_vec(), record.target().get_buffer("buffer")?.data);

        Ok(())
    }

    #[test]
    fn test_action_fails_on_bad_change() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change some middle bytes so we can test undo
        record.apply(Action::buffer_edit("buffer", b"BBBBBB".to_vec(), 2))?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change one too many bytes from the start
        assert!(record.apply(Action::buffer_edit("buffer", b"AAAAAAAAAAB".to_vec(), 0)).is_err());
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change one too many bytes from the end
        assert!(record.apply(Action::buffer_edit("buffer", b"AB".to_vec(), 9)).is_err());
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change something completely off the end
        assert!(record.apply(Action::buffer_edit("buffer", b"AB".to_vec(), 100)).is_err());
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Make sure undo and redo are correct
        record.undo()?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);
        record.redo()?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        Ok(())
    }

    #[test]
    fn test_action_fails_on_zero_size() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change some middle bytes so we can test undo
        record.apply(Action::buffer_edit("buffer", b"BBBBBB".to_vec(), 2))?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Change zero bytes, make sure it fails
        assert!(record.apply(Action::buffer_edit("buffer", b"".to_vec(), 0)).is_err());
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Make sure undo and redo are correct
        record.undo()?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);
        record.redo()?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);

        Ok(())
    }
}
