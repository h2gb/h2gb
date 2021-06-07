//! Change the name of a buffer.
//!
//! Currently, this will not work on a populated buffer. In the future, we might
//! want to make that possible, but it'll require "fixing" broken links.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferRenameForward {
    pub original_name: String,
    pub new_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferRenameBackward {
    original_name: String,
    new_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferRename {
    forward: Option<ActionBufferRenameForward>,
    backward: Option<ActionBufferRenameBackward>,
}

impl ActionBufferRename {
    pub fn new(forward: ActionBufferRenameForward) -> Self {
        ActionBufferRename {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, &str)> for ActionBufferRename {
    fn from(o: (&str, &str)) -> Self {
        ActionBufferRename {
            forward: Some(ActionBufferRenameForward {
                original_name: o.0.to_string(),
                new_name: o.1.to_string(),
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferRename {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Do the rename
        project.buffer_rename(&forward.original_name, &forward.new_name)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferRenameBackward {
            original_name: forward.original_name.clone(),
            new_name: forward.new_name.clone(),
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        // Do the rename inverted
        project.buffer_rename(&backward.new_name, &backward.original_name)?;

        self.forward = Some(ActionBufferRenameForward {
            original_name: backward.original_name.clone(),
            new_name: backward.new_name.clone(),
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
        assert_eq!(true, record.target().buffer_exists("buffer"));
        assert_eq!(false, record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("newerbuffer"));

        // Rename a couple times
        record.apply(Action::buffer_rename("buffer", "newbuffer"))?;
        assert_eq!(false, record.target().buffer_exists("buffer"));
        assert_eq!(true, record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("newerbuffer"));

        record.apply(Action::buffer_rename("newbuffer", "newerbuffer"))?;
        assert_eq!(false, record.target().buffer_exists("buffer"));
        assert_eq!(false, record.target().buffer_exists("newbuffer"));
        assert_eq!(true, record.target().buffer_exists("newerbuffer"));

        record.undo()?;
        assert_eq!(false, record.target().buffer_exists("buffer"));
        assert_eq!(true, record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("newerbuffer"));

        record.undo()?;
        assert_eq!(true, record.target().buffer_exists("buffer"));
        assert_eq!(false, record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("newerbuffer"));

        record.redo()?;
        assert_eq!(false, record.target().buffer_exists("buffer"));
        assert_eq!(true, record.target().buffer_exists("newbuffer"));
        assert_eq!(false, record.target().buffer_exists("newerbuffer"));

        record.redo()?;
        assert_eq!(false, record.target().buffer_exists("buffer"));
        assert_eq!(false, record.target().buffer_exists("newbuffer"));
        assert_eq!(true, record.target().buffer_exists("newerbuffer"));

        Ok(())
    }

    #[test]
    fn test_action_fails_on_conflict() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create two buffers
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;
        record.apply(Action::buffer_create_from_bytes("conflictingbuffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;

        // Fail to rename
        assert!(record.apply(Action::buffer_rename("buffer", "conflictingbuffer")).is_err());

        Ok(())
    }

    #[test]
    fn test_action_fails_on_missing_source() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Fail to rename a buffer that doesn't exist
        assert!(record.apply(Action::buffer_rename("buffer", "newbuffer")).is_err());

        Ok(())
    }

    #[test]
    fn test_action_fails_on_blank_name() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;

        // Fail to rename to nothing
        assert!(record.apply(Action::buffer_rename("buffer", "")).is_err());

        Ok(())
    }
}
