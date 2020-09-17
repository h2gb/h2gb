use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::{H2Project, H2Buffer};

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

impl Command for ActionBufferCreateEmpty {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match self.forward.take() {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Sanity check
        if project.buffer_exists(&forward.name) {
            bail!("A buffer with that name already exists");
        }

        // Apply the change
        let buffer = H2Buffer::new(vec![0; forward.size], forward.base_address);
        project.buffer_insert(&forward.name, buffer)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferCreateEmptyBackward {
            name: forward.name,
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match self.backward.take() {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        let name = backward.name;
        let buffer = project.buffer_remove(&name)?;

        self.forward = Some(ActionBufferCreateEmptyForward {
            name: name,
            size: buffer.data.len(),
            base_address: buffer.base_address,
        });

        Ok(())
    }
}
