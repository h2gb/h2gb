use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::h2project::H2Project;

pub mod null;
pub mod change_project_name;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Null(null::NullAction),
    ChangeProjectName(change_project_name::ActionChangeProjectName),
}

impl Command for Action {
    type Target = H2Project;
    type Error = SimpleResult<()>;

    fn apply(&mut self, project: &mut H2Project) -> redo::Result<Self> {
        match self {
            Action::Null(a) => a.apply(project),
            Action::ChangeProjectName(a) => a.apply(project),
        }
    }

    fn undo(&mut self, project: &mut H2Project) -> redo::Result<Self> {
        match self {
            Action::Null(a) => a.undo(project),
            Action::ChangeProjectName(a) => a.undo(project),
        }
    }
}
