//! Create a new buffer of a particular size, initialized to all zeroes.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::project::h2project::H2Project;
use crate::project::h2buffer::H2Buffer;
use crate::project::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    pub name: String,
    pub size: usize,
    pub base_address: usize,
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
pub struct ActionBufferCreateEmpty {
    state: State,
}

impl ActionBufferCreateEmpty {
    pub fn new(name: &str, size: usize, base_address: usize) -> Action {
        let state = State::Forward(Forward {
            name: String::from(name),
            size: size,
            base_address: base_address,
        });

        Action::BufferCreateEmpty(ActionBufferCreateEmpty { state: state } )
    }
}

impl Command for ActionBufferCreateEmpty {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.state {
            State::Forward(f) => f,
            _                   => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Apply the change
        let buffer = H2Buffer::new(vec![0; forward.size], forward.base_address)?;
        project.buffer_insert(&forward.name, buffer)?;

        // Swap backward + forward
        self.state = State::Backward(Backward {
            name: forward.name.to_string(),
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.state {
            State::Backward(b) => b,
            _                    => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        let name = &backward.name;
        let buffer = project.buffer_remove(name)?;

        self.state = State::Forward(Forward {
            name: name.clone(),
            size: buffer.data.len(),
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

        let action = ActionBufferCreateEmpty::new("buffer", 10, 0x80000000);
        record.apply(action)?;

        let buffers = record.target().buffers();
        assert_eq!(1, buffers.len());
        assert_eq!(10, buffers["buffer"].data.len());
        assert_eq!(0x80000000, buffers["buffer"].base_address);

        record.undo()?;
        record.redo()?;

        let buffers = record.target().buffers();
        assert_eq!(1, buffers.len());
        assert_eq!(10, buffers["buffer"].data.len());
        assert_eq!(0x80000000, buffers["buffer"].base_address);

        Ok(())
    }

    #[test]
    fn test_action_fails_if_buffer_already_exists() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(ActionBufferCreateEmpty::new("buffer", 10, 0x80000000)).is_ok());
        assert!(record.apply(ActionBufferCreateEmpty::new("buffer2", 10, 0x80000000)).is_ok());
        assert!(record.apply(ActionBufferCreateEmpty::new("buffer", 10, 0x80000000)).is_err());
        assert!(record.apply(ActionBufferCreateEmpty::new("buffer2", 10, 0x80000000)).is_err());

        Ok(())
    }

    #[test]
    fn test_action_fails_with_zero_size() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert!(record.apply(ActionBufferCreateEmpty::new("buffer", 0, 0x80000000)).is_err());

        Ok(())
    }
}
