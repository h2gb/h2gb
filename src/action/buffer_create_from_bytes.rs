//! Create a new buffer consisting of the provided data. Will commonly be read
//! from a file.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;
use crate::h2buffer::H2Buffer;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCreateFromBytesForward {
    pub name: String,
    pub data: Vec<u8>,
    pub base_address: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferCreateFromBytesBackward {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCreateFromBytes {
    forward: Option<ActionBufferCreateFromBytesForward>,
    backward: Option<ActionBufferCreateFromBytesBackward>,
}

impl ActionBufferCreateFromBytes {
    pub fn new(forward: ActionBufferCreateFromBytesForward) -> Self {
        ActionBufferCreateFromBytes {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, Vec<u8>, usize)> for ActionBufferCreateFromBytes {
    fn from(o: (&str, Vec<u8>, usize)) -> Self {
        ActionBufferCreateFromBytes {
            forward: Some(ActionBufferCreateFromBytesForward {
                name: o.0.to_string(),
                data: o.1,
                base_address: o.2,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferCreateFromBytes {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Sanity check: it has a size
        // Apply the change
        let buffer = H2Buffer::new(forward.data.clone(), forward.base_address)?;
        project.buffer_insert(&forward.name, buffer)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferCreateFromBytesBackward {
            name: forward.name.clone(),
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

        self.forward = Some(ActionBufferCreateFromBytesForward {
            name: name.clone(),
            data: buffer.data,
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

    use crate::h2project::H2Project;
    use redo::Record;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<ActionBufferCreateFromBytes> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert_eq!(0, record.target().buffers().len());

        record.apply(("buffer", vec![0, 1, 2, 4], 0x80000000).into())?;

        let buffers = record.target().buffers();
        assert_eq!(1, buffers.len());
        assert_eq!(4, buffers["buffer"].data.len());
        assert_eq!(vec![0, 1, 2, 4], buffers["buffer"].data);
        assert_eq!(0x80000000, buffers["buffer"].base_address);

        record.undo()?;
        record.redo()?;

        Ok(())
    }

    #[test]
    fn test_action_fails_if_buffer_already_exists() -> SimpleResult<()> {
        let mut record: Record<ActionBufferCreateFromBytes> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(("buffer", vec![0, 1, 2, 4], 0x80000000).into()).is_ok());
        assert!(record.apply(("buffer123", vec![0, 1, 2, 4], 0x80000000).into()).is_ok());
        assert!(record.apply(("buffer123", vec![0, 1, 2, 4], 0x80000000).into()).is_err());
        assert!(record.apply(("buffer", vec![0, 1, 2, 4], 0x80000000).into()).is_err());

        Ok(())
    }

    #[test]
    fn test_action_fails_with_zero_size() -> SimpleResult<()> {
        let mut record: Record<ActionBufferCreateFromBytes> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(("buffer", vec![], 0x80000000).into()).is_err());

        Ok(())
    }
}
