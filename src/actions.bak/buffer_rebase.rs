//! Change the base_address of a buffer.
//!
//! This is currently very simple; the address changes and that's it. In the
//! future, we may take more actions (like letting layers / entries know that
//! something changed).

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferRebaseForward {
    pub buffer_name: String,
    pub new_base_address: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferRebaseBackward {
    buffer_name: String,
    old_base_address: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferRebase {
    forward: Option<ActionBufferRebaseForward>,
    backward: Option<ActionBufferRebaseBackward>,
}

impl ActionBufferRebase {
    pub fn new(forward: ActionBufferRebaseForward) -> Self {
        ActionBufferRebase {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, usize)> for ActionBufferRebase {
    fn from(o: (&str, usize)) -> Self {
        ActionBufferRebase {
            forward: Some(ActionBufferRebaseForward {
                buffer_name: o.0.to_string(),
                new_base_address: o.1,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferRebase {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Get the buffer
        let b = project.get_buffer_mut(&forward.buffer_name)?;

        // Rebase it
        let old_base_address = b.rebase(forward.new_base_address)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferRebaseBackward {
            buffer_name: forward.buffer_name.clone(),
            old_base_address: old_base_address,
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        // Get the buffer
        let b = project.get_buffer_mut(&backward.buffer_name)?;

        // Rebase it
        let new_base_address = b.rebase(backward.old_base_address)?;

        self.forward = Some(ActionBufferRebaseForward {
            buffer_name: backward.buffer_name.clone(),
            new_base_address: new_base_address
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
        assert_eq!(0x80000000, record.target().get_buffer("buffer")?.base_address);

        // Change the base address a couple times
        record.apply(Action::buffer_rebase("buffer", 0x0))?;
        assert_eq!(0x0, record.target().get_buffer("buffer")?.base_address);

        record.apply(Action::buffer_rebase("buffer", 0xFFFFFFFF))?;
        assert_eq!(0xFFFFFFFF, record.target().get_buffer("buffer")?.base_address);

        // Undo / redo
        record.undo()?;
        assert_eq!(0x0, record.target().get_buffer("buffer")?.base_address);

        record.undo()?;
        assert_eq!(0x80000000, record.target().get_buffer("buffer")?.base_address);

        record.redo()?;
        assert_eq!(0x0, record.target().get_buffer("buffer")?.base_address);

        record.redo()?;
        assert_eq!(0xFFFFFFFF, record.target().get_buffer("buffer")?.base_address);

        Ok(())
    }

    #[test]
    fn test_action_fails_on_missing_buffer() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // No buffer is created
        assert!(record.apply(Action::buffer_rebase("buffer", 0x0)).is_err());

        Ok(())
    }
}
