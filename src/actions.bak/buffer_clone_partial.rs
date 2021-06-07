//! An action that clones a buffer's data to a new name.
//!
//! Preserves the data and base address, but nothing else. Can be edited and
//! stuff, like any buffer, but can't be merged back (since it's a copy).
//!
//! Unless a custom base_address is passed to the object, this uses the offset
//! of the cloned data as the base_address (in other words, if the buffer's
//! base_address is 0x1000, and you copy starting at byte 8, the new
//! base_address will be 0x1008).

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};
use std::ops::Range;

use crate::project::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferClonePartialForward {
    /// The name of the source buffer - obviously, must exist as a buffer in
    /// the project
    pub clone_from_name: String,

    /// The name of the new buffer - must NOT exist as a buffer in the project
    pub clone_to_name: String,

    /// The range of bytes to clone. This must be within the buffer, but may
    /// be the entire buffer
    pub range: Range<usize>,

    /// Optionally set a new base_address value; by default, will choose the
    /// buffer's current base_address + `range.start`
    pub new_base_address: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferClonePartialBackward {
    clone_from_name: String,
    clone_to_name: String,
    range: Range<usize>,
    new_base_address: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferClonePartial {
    forward: Option<ActionBufferClonePartialForward>,
    backward: Option<ActionBufferClonePartialBackward>,
}

impl ActionBufferClonePartial {
    pub fn new(forward: ActionBufferClonePartialForward) -> Self {
        ActionBufferClonePartial {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, &str, Range<usize>, Option<usize>)> for ActionBufferClonePartial {
    fn from(o: (&str, &str, Range<usize>, Option<usize>)) -> Self {
        ActionBufferClonePartial {
            forward: Some(ActionBufferClonePartialForward {
                clone_from_name: o.0.to_string(),
                clone_to_name: o.1.to_string(),
                range: o.2,
                new_base_address: o.3,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferClonePartial {
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

        // Apply the change
        let original = project.get_buffer(&forward.clone_from_name)?;
        let new_buffer = original.clone_partial(forward.range.clone(), forward.new_base_address)?;
        project.buffer_insert(&forward.clone_to_name, new_buffer)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferClonePartialBackward {
            clone_to_name: forward.clone_to_name.clone(),
            clone_from_name: forward.clone_from_name.clone(),
            range: forward.range.clone(),
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

        self.forward = Some(ActionBufferClonePartialForward {
            clone_to_name: backward.clone_to_name.clone(),
            clone_from_name: backward.clone_from_name.clone(),
            range: backward.range.clone(),
            new_base_address: backward.new_base_address,
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
        record.apply(Action::buffer_create_from_bytes("buffer", b"AABBCCDDEE".to_vec(), 0x80000000))?;
        assert_eq!(b"AABBCCDDEE".to_vec(), record.target().get_buffer("buffer")?.data);
        assert_eq!(0x80000000, record.target().get_buffer("buffer")?.base_address);

        // Clone the middle of it
        record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2..8, None))?;
        assert_eq!(b"BBCCDD".to_vec(), record.target().get_buffer("newbuffer")?.data);

        // Clone the end of that
        record.apply(Action::buffer_clone_partial("newbuffer", "othernewbuffer", 4..6, None))?;
        assert_eq!(b"DD".to_vec(), record.target().get_buffer("othernewbuffer")?.data);

        // Test undo / redo
        record.undo()?;
        assert_eq!(true,  record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("othernewbuffer"));

        record.undo()?;
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("othernewbuffer"));

        record.redo()?;
        assert_eq!(true,  record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("othernewbuffer"));
        assert_eq!(b"BBCCDD".to_vec(), record.target().get_buffer("newbuffer")?.data);

        record.redo()?;
        assert_eq!(true,  record.target().buffer_exists("newbuffer"));
        assert_eq!(true,  record.target().buffer_exists("othernewbuffer"));
        assert_eq!(b"BBCCDD".to_vec(), record.target().get_buffer("newbuffer")?.data);
        assert_eq!(b"DD".to_vec(), record.target().get_buffer("othernewbuffer")?.data);

        Ok(())
    }

    #[test]
    fn test_base_address() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AABBCCDDEE".to_vec(), 0x80000000))?;
        assert_eq!(b"AABBCCDDEE".to_vec(), record.target().get_buffer("buffer")?.data);
        assert_eq!(0x80000000, record.target().get_buffer("buffer")?.base_address);

        // Make sure it uses the "right" default base address
        record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2..8, None))?;
        assert_eq!(0x80000002, record.target().get_buffer("newbuffer")?.base_address);

        // Try a custom base address
        record.apply(Action::buffer_clone_partial("buffer", "newbuffer2", 2..8, Some(0)))?;
        assert_eq!(0, record.target().get_buffer("newbuffer2")?.base_address);

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

        // Clone one too many bytes from the start
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 0..11, None)).is_err());
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));

        // Clone one too many bytes from the end
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2..11, None)).is_err());
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));

        // Change something completely off the end
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2..100, None)).is_err());
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));

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

        // Clone one too many bytes from the start
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 0..0, None)).is_err());
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2..2, None)).is_err());
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 100..100, None)).is_err());
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));

        Ok(())
    }
}
