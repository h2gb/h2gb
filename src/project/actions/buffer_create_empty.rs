//! Create a new buffer of a particular size, initialized to all zeroes.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;
use crate::project::h2buffer::H2Buffer;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCreateEmptyForward {
    pub name: String,
    pub size: usize,
    pub base_address: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferCreateEmptyBackward {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCreateEmpty {
    forward: Option<ActionBufferCreateEmptyForward>,
    backward: Option<ActionBufferCreateEmptyBackward>,
}

impl ActionBufferCreateEmpty {
    pub fn new(forward: ActionBufferCreateEmptyForward) -> Self {
        ActionBufferCreateEmpty {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, usize, usize)> for ActionBufferCreateEmpty {
    fn from(o: (&str, usize, usize)) -> Self {
        ActionBufferCreateEmpty {
            forward: Some(ActionBufferCreateEmptyForward {
                name: o.0.to_string(),
                size: o.1,
                base_address: o.2,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferCreateEmpty {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Apply the change
        let buffer = H2Buffer::new(vec![0; forward.size], forward.base_address)?;
        project.buffer_insert(&forward.name, buffer)?;

        // Swap backward + forward
        self.backward = Some(ActionBufferCreateEmptyBackward {
            name: forward.name.to_string(),
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        let name = &backward.name;
        let buffer = project.buffer_remove(name)?;

        self.forward = Some(ActionBufferCreateEmptyForward {
            name: name.clone(),
            size: buffer.data.len(),
            base_address: buffer.base_address,
        });
        self.backward = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    use crate::project::h2project::H2Project;
    use redo::Record;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<ActionBufferCreateEmpty> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert_eq!(0, record.target().buffers().len());

        record.apply(("buffer", 10, 0x80000000).into())?;

        let buffers = record.target().buffers();
        assert_eq!(1, buffers.len());

        assert_eq!(10, buffers["buffer"].data.len());
        assert_eq!(0x80000000, buffers["buffer"].base_address);

        record.undo()?;

        record.redo()?;

        Ok(())
    }

    #[test]
    fn test_action_fails_if_buffer_already_exists() -> SimpleResult<()> {
        let mut record: Record<ActionBufferCreateEmpty> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(("buffer", 10, 0x80000000).into()).is_ok());
        assert!(record.apply(("buffer2", 10, 0x80000000).into()).is_ok());
        assert!(record.apply(("buffer", 10, 0x80000000).into()).is_err());
        assert!(record.apply(("buffer2", 10, 0x80000000).into()).is_err());

        Ok(())
    }

    #[test]
    fn test_action_fails_with_zero_size() -> SimpleResult<()> {
        let mut record: Record<ActionBufferCreateEmpty> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(("buffer", 0, 0x80000000).into()).is_err());

        Ok(())
    }
}
