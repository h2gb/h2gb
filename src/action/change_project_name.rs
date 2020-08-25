use std::mem;

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionChangeProjectName {
    pub new_name: String,
    original_name: Option<String>,
}

impl ActionChangeProjectName {
    pub fn new(new_name: &str) -> Self {
        ActionChangeProjectName {
            new_name: String::from(new_name),
            original_name: None,
        }
    }
}

impl Command for ActionChangeProjectName {
    type Target = H2Project;
    type Error = SimpleResult<()>;

    fn apply(&mut self, project: &mut H2Project) -> redo::Result<Self> {
        self.original_name = Some(mem::replace(&mut project.name, self.new_name.clone()));

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> redo::Result<Self> {
        project.name = self.original_name.clone().unwrap_or(String::from("Couldn't determine name"));
        self.original_name = None;

        Ok(())
    }
}
