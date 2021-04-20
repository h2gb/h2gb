// An enum that combines all possible actions.
//
// Most of the methods here are simply wrappers for the actual action, which
// are all defined in their respective modules. For documentation, see them.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError};

use crate::project::h2project::H2Project;

// Still needed:
// * Layers
//   * layer_create
//   * layer_delete
//   * layer_rename
//
// * Entries
//   * entry_create
//   * entry_create_multiple
//   * entry_delete
//   * entry_unlink
//   * entry_recompute / entry_rebase (maybe?)
//   * entry_edit
//
// * create_buffer_from_entry
//
// * Are references separate?
// * What about pointers / structs / arrays / etc?

pub mod buffer_create_empty;
pub mod null;

use buffer_create_empty::*;
use null::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Null(null::NullAction),
    BufferCreateEmpty(buffer_create_empty::ActionBufferCreateEmpty),
}

impl Action {
    // Not sure this is possible
    // fn get_command(&self) -> Box<dyn Command<Target = H2Project, Error = SimpleError>> {
    //     match self {
    //         Action::Null(a)              => Box::new(*a),
    //         Action::BufferCreateEmpty(a) => Box::new(*a),
    //     }
    // }
}

impl Command for Action {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        match self {
            Action::Null(a) => a.apply(project),
            Action::BufferCreateEmpty(a) => a.apply(project),
        }
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        match self {
            Action::Null(a) => a.undo(project),
            Action::BufferCreateEmpty(a) => a.undo(project),
        }
    }
}
