// An enum that combines all possible actions.
//
// Most of the methods here are simply wrappers for the actual action, which
// are all defined in their respective modules. For documentation, see them.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError};

use crate::project::H2Project;

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

mod buffer_create_empty;
pub use buffer_create_empty::ActionBufferCreateEmpty;

mod buffer_create_from_bytes;
pub use buffer_create_from_bytes::ActionBufferCreateFromBytes;

mod buffer_extract;
pub use buffer_extract::ActionBufferExtract;

mod buffer_transform;
pub use buffer_transform::ActionBufferTransform;

mod null;
pub use null::NullAction;

mod layer_create;
pub use layer_create::ActionLayerCreate;

// mod entry_create_and_insert;
// pub use entry_create_and_insert::ActionEntryCreateAndInsert;

mod entry_create;
pub use entry_create::ActionEntryCreate;

mod entry_set_comment;
pub use entry_set_comment::ActionEntrySetComment;

// Don't create this directly - use the actions' new() functions
#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Null(NullAction),
    BufferCreateEmpty(ActionBufferCreateEmpty),
    BufferCreateFromBytes(ActionBufferCreateFromBytes),
    BufferExtract(ActionBufferExtract),
    BufferTransform(ActionBufferTransform),
    LayerCreate(ActionLayerCreate),
    // EntryCreateAndInsert(ActionEntryCreateAndInsert),
    EntryCreate(ActionEntryCreate),
    EntrySetComment(ActionEntrySetComment),
}

impl Command for Action {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        match self {
            Action::Null(a)                  => a.apply(project),
            Action::BufferCreateEmpty(a)     => a.apply(project),
            Action::BufferCreateFromBytes(a) => a.apply(project),
            Action::BufferExtract(a)         => a.apply(project),
            Action::BufferTransform(a)       => a.apply(project),
            Action::LayerCreate(a)           => a.apply(project),
            // Action::EntryCreateAndInsert(a)  => a.apply(project),
            Action::EntryCreate(a)           => a.apply(project),
            Action::EntrySetComment(a)       => a.apply(project),
        }
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        match self {
            Action::Null(a)                  => a.undo(project),
            Action::BufferCreateEmpty(a)     => a.undo(project),
            Action::BufferCreateFromBytes(a) => a.undo(project),
            Action::BufferExtract(a)         => a.undo(project),
            Action::BufferTransform(a)       => a.undo(project),
            Action::LayerCreate(a)           => a.undo(project),
            // Action::EntryCreateAndInsert(a)  => a.undo(project),
            Action::EntryCreate(a)           => a.undo(project),
            Action::EntrySetComment(a)       => a.undo(project),
        }
    }
}
