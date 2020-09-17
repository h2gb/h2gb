use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::{H2Project, H2Buffer};

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferDeleteForward {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferDeleteBackward {
    name: String,
    data: Vec<u8>,
    base_address: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferDelete {
    forward: Option<ActionBufferDeleteForward>,
    backward: Option<ActionBufferDeleteBackward>,
}

impl ActionBufferDelete {
    pub fn new(forward: ActionBufferDeleteForward) -> Self {
        ActionBufferDelete {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl Command for ActionBufferDelete {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let forward = match self.forward.take() {
            Some(b) => b,
            None => bail!("Failed to apply: missing context"),
        };

        let name = forward.name;
        let buffer = project.buffer_remove(&name)?;

        self.backward = Some(ActionBufferDeleteBackward {
            name: name,
            data: buffer.data,
            base_address: buffer.base_address,
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match self.backward.take() {
            Some(f) => f,
            None => bail!("Failed to undo: missing context"),
        };

        let buffer = H2Buffer::new(backward.data, backward.base_address);
        project.buffer_insert(&backward.name, buffer)?;

        self.forward = Some(ActionBufferDeleteForward {
            name: backward.name,
        });

        Ok(())
    }
}
