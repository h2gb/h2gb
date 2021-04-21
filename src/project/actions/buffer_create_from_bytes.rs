//! Create a new buffer of a particular size, initialized to all zeroes.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;
use crate::project::h2buffer::H2Buffer;
use crate::project::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    name: String,
    data: Vec<u8>,
    base_address: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Backward {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCreateFromBytes(State);

impl ActionBufferCreateFromBytes {
    pub fn new(name: &str, data: &[u8], base_address: usize) -> Action {
        Action::BufferCreateFromBytes(
            ActionBufferCreateFromBytes(
                State::Forward(Forward {
                    name: String::from(name),
                    data: Vec::from(data),
                    base_address: base_address,
                })
            )
        )
    }
}

impl Command for ActionBufferCreateFromBytes {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        let buffer = H2Buffer::new(forward.data.clone(), forward.base_address)?;
        project.buffer_insert(&forward.name, buffer)?;

        // Save the backward struct
        self.0 = State::Backward(Backward {
            name: forward.name.to_string(),
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the backward struct
        let backward = match &self.0 {
            State::Backward(b) => b,
            _                    => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        let name = &backward.name;
        let buffer = project.buffer_remove(name)?;

        // Save the forward struct
        self.0 = State::Forward(Forward {
            name: name.clone(),
            data: buffer.data,
            base_address: buffer.base_address,
        });

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
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert_eq!(0, record.target().buffers().len());

        let action = ActionBufferCreateFromBytes::new(
            "buffer",
            &vec![0, 1, 2, 4],
            0x80000000,
        );
        record.apply(action)?;

        let buffers = record.target().buffers();
        assert_eq!(1, buffers.len());
        assert_eq!(4, buffers["buffer"].data.len());
        assert_eq!(vec![0, 1, 2, 4], buffers["buffer"].data);
        assert_eq!(0x80000000, buffers["buffer"].base_address);

        record.undo()?;
        record.redo()?;

        let buffers = record.target().buffers();
        assert_eq!(1, buffers.len());
        assert_eq!(4, buffers["buffer"].data.len());
        assert_eq!(vec![0, 1, 2, 4], buffers["buffer"].data);
        assert_eq!(0x80000000, buffers["buffer"].base_address);

        Ok(())
    }

    #[test]
    fn test_action_fails_if_buffer_already_exists() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(ActionBufferCreateFromBytes::new("buffer",    &vec![0, 1, 2, 4], 0x80000000)).is_ok());
        assert!(record.apply(ActionBufferCreateFromBytes::new("buffer123", &vec![0, 1, 2, 4], 0x80000000)).is_ok());
        assert!(record.apply(ActionBufferCreateFromBytes::new("buffer123", &vec![0, 1, 2, 4], 0x80000000)).is_err());
        assert!(record.apply(ActionBufferCreateFromBytes::new("buffer",    &vec![0, 1, 2, 4], 0x80000000)).is_err());

        Ok(())
    }

    #[test]
    fn test_action_fails_with_zero_size() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(ActionBufferCreateFromBytes::new("buffer", &vec![], 0x80000000)).is_err());

        Ok(())
    }
}
