use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;

// TODO: Change start / size to a Range
#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferClonePartialForward {
    pub clone_from_name: String,
    pub clone_to_name: String,
    pub start: usize,
    pub size: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferClonePartialBackward {
    clone_from_name: String,
    clone_to_name: String,
    pub start: usize,
    pub size: usize,
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

impl From<(&str, &str, usize, usize)> for ActionBufferClonePartial {
    fn from(o: (&str, &str, usize, usize)) -> Self {
        ActionBufferClonePartial {
            forward: Some(ActionBufferClonePartialForward {
                clone_from_name: o.0.to_string(),
                clone_to_name: o.1.to_string(),
                start: o.2,
                size: o.3,
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

        // Apply the change
        project.buffer_clone_partial(&forward.clone_from_name, &forward.clone_to_name, forward.start, forward.size)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferClonePartialBackward {
            clone_to_name: forward.clone_to_name.clone(),
            clone_from_name: forward.clone_from_name.clone(),
            start: forward.start,
            size: forward.size,
        });

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
            start: backward.start,
            size: backward.size,
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

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AABBCCDDEE".to_vec(), 0x80000000))?;
        assert_eq!(b"AABBCCDDEE".to_vec(), record.target().get_buffer("buffer")?.data);
        assert_eq!(0x80000000, record.target().get_buffer("buffer")?.base_address);

        // Clone the middle of it
        record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2, 6))?;
        assert_eq!(b"BBCCDD".to_vec(), record.target().get_buffer("newbuffer")?.data);

        // Clone the end of that
        record.apply(Action::buffer_clone_partial("newbuffer", "othernewbuffer", 4, 2))?;
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
    fn test_action_fails_on_bad_change() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);

        // Clone one too many bytes from the start
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 0, 11)).is_err());
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));

        // Clone one too many bytes from the end
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2, 9)).is_err());
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));

        // Change something completely off the end
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2, 100)).is_err());
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
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 0, 0)).is_err());
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 2, 0)).is_err());
        assert!(record.apply(Action::buffer_clone_partial("buffer", "newbuffer", 100, 0)).is_err());
        assert_eq!(false,  record.target().buffer_exists("newbuffer"));

        Ok(())
    }
}
