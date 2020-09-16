use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionProjectRenameForward {
    new_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionProjectRenameBackward {
    old_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionProjectRename {
    forward: Option<ActionProjectRenameForward>,
    backward: Option<ActionProjectRenameBackward>,
}

impl ActionProjectRename {
    pub fn new(new_name: &str) -> Self {
        ActionProjectRename {
            forward: Some(ActionProjectRenameForward {
                new_name: new_name.to_string(),
            }),
            backward: None,
        }
    }
}

impl From<&str> for ActionProjectRename {
    fn from(o: &str) -> Self {
        ActionProjectRename::new(o)
    }
}

impl Command for ActionProjectRename {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let forward = match self.forward.take() {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        let old_name = project.project_rename(forward.new_name)?;

        self.backward = Some(ActionProjectRenameBackward {
            old_name: old_name
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match self.backward.take() {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        let new_name = project.project_rename(backward.old_name)?;

        self.forward = Some(ActionProjectRenameForward {
            new_name: new_name
        });

        Ok(())
    }
}
