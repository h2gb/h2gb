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
pub use buffer_create_empty::ActionBufferCreateEmpty;

pub mod buffer_create_from_bytes;
pub use buffer_create_from_bytes::ActionBufferCreateFromBytes;

pub mod buffer_transform;
pub use buffer_transform::ActionBufferTransform;

pub mod null;
pub use null::NullAction;

// Don't create this directly - use the actions' new() functions
#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Null(NullAction),
    BufferCreateEmpty(ActionBufferCreateEmpty),
    BufferCreateFromBytes(ActionBufferCreateFromBytes),
    BufferTransform(ActionBufferTransform),
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
            Action::Null(a)                  => a.apply(project),
            Action::BufferCreateEmpty(a)     => a.apply(project),
            Action::BufferCreateFromBytes(a) => a.apply(project),
            Action::BufferTransform(a)       => a.apply(project),
        }
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        match self {
            Action::Null(a)                  => a.undo(project),
            Action::BufferCreateEmpty(a)     => a.undo(project),
            Action::BufferCreateFromBytes(a) => a.undo(project),
            Action::BufferTransform(a)       => a.undo(project),
        }
    }
}
