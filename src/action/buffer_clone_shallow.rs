//! An action that clones part of a buffer to a new name.
//!
//! Preserves the data and base address, but nothing else. Can be edited and
//! stuff, like any buffer, but can't be merged back (since it's a copy).
//!
//! Copies the original base_address unless a custom one is passed in.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCloneShallowForward {
    pub clone_from_name: String,
    pub clone_to_name: String,
    pub new_base_address: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferCloneShallowBackward {
    clone_from_name: String,
    clone_to_name: String,
    new_base_address: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCloneShallow {
    forward: Option<ActionBufferCloneShallowForward>,
    backward: Option<ActionBufferCloneShallowBackward>,
}

impl ActionBufferCloneShallow {
    pub fn new(forward: ActionBufferCloneShallowForward) -> Self {
        ActionBufferCloneShallow {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, &str, Option<usize>)> for ActionBufferCloneShallow {
    fn from(o: (&str, &str, Option<usize>)) -> Self {
        ActionBufferCloneShallow {
            forward: Some(ActionBufferCloneShallowForward {
                clone_from_name: o.0.to_string(),
                clone_to_name: o.1.to_string(),
                new_base_address: o.2,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferCloneShallow {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Make sure we'll be able to insert - this is mostly to save the
        // cloning time
        if project.buffer_exists(&forward.clone_to_name) {
            bail!("Buffer already exists: {}", forward.clone_to_name);
        }

        let original = project.get_buffer(&forward.clone_from_name)?;
        let new_buffer = original.clone_shallow(forward.new_base_address)?;
        project.buffer_insert(&forward.clone_to_name, new_buffer)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferCloneShallowBackward {
            clone_to_name: forward.clone_to_name.clone(),
            clone_from_name: forward.clone_from_name.clone(),
            new_base_address: forward.new_base_address,
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        // We don't need to keep a copy of the buffer since the name is enough
        // to clone it
        project.buffer_remove(&backward.clone_to_name)?;

        self.forward = Some(ActionBufferCloneShallowForward {
            clone_to_name: backward.clone_to_name.clone(),
            clone_from_name: backward.clone_from_name.clone(),
            new_base_address: backward.new_base_address,
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

    use crate::transformation::Transformation;

    use crate::action::Action;
    use crate::h2project::H2Project;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Clone it
        record.apply(Action::buffer_clone_shallow("buffer", "newbuffer", None))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("newbuffer")?.data);

        // Edit the original and ensure the change isn't reflected
        record.apply(Action::buffer_edit("buffer", b"BBBBBB".to_vec(), 2))?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("newbuffer")?.data);

        // Undo the edit
        record.undo()?;

        // Undo the clone
        record.undo()?;
        assert_eq!(false, record.target().buffer_exists("newbuffer"));
        assert!(record.target().get_buffer("newbuffer").is_err());

        record.redo()?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("newbuffer")?.data);

        Ok(())
    }

    #[test]
    fn test_base_address() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;

        // Clone it with no base address, it should have the same one
        record.apply(Action::buffer_clone_shallow("buffer", "newbuffer", None))?;
        assert_eq!(0x80000000, record.target().get_buffer("newbuffer")?.base_address);

        // Clone it again, this time customize the base address
        record.apply(Action::buffer_clone_shallow("buffer", "newbuffer2", Some(10)))?;
        assert_eq!(10, record.target().get_buffer("newbuffer2")?.base_address);

        Ok(())
    }

    #[test]
    fn test_action_fails_on_bad_change() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;

        // Rename it to itself (invalid)
        assert!(record.apply(Action::buffer_clone_shallow("buffer", "buffer", None)).is_err());

        Ok(())
    }

    #[test]
    fn test_untransform_is_preserved() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a simple buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"4a4B4c4D4e".to_vec(), 0x80000000))?;

        // Transform it
        record.apply(Action::buffer_transform("buffer", Transformation::FromHex))?;

        // Clone
        record.apply(Action::buffer_clone_shallow("buffer", "newbuffer", None))?;
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("newbuffer")?.data);

        // Ensure that untransform works on the new buffer
        record.apply(Action::buffer_untransform("newbuffer"))?;
        assert_eq!(b"4a4b4c4d4e".to_vec(), record.target().get_buffer("newbuffer")?.data);
        assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

        Ok(())
    }
}
