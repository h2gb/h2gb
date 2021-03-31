//! An enum that combines all possible actions.
//!
//! Most of the methods here are simply wrappers for the actual action, which
//! are all defined in their respective modules. For documentation, see them.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError};
use std::ops::Range;

use crate::transformation::H2Transformation;
use crate::h2project::H2Project;

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
// * Are references separate?
// * What about pointers / structs / arrays / etc?

pub mod buffer_clone_partial;
pub mod buffer_clone_shallow;
pub mod buffer_create_empty;
pub mod buffer_create_from_bytes;
pub mod buffer_delete;
pub mod buffer_edit;
pub mod buffer_rebase;
pub mod buffer_rename;
pub mod buffer_split;
pub mod buffer_transform;
pub mod buffer_untransform;
pub mod null;
pub mod project_rename;

use buffer_clone_partial::{ActionBufferClonePartial, ActionBufferClonePartialForward};
use buffer_clone_shallow::{ActionBufferCloneShallow, ActionBufferCloneShallowForward};
use buffer_create_empty::{ActionBufferCreateEmpty, ActionBufferCreateEmptyForward};
use buffer_create_from_bytes::{ActionBufferCreateFromBytes, ActionBufferCreateFromBytesForward};
use buffer_delete::{ActionBufferDelete, ActionBufferDeleteForward};
use buffer_edit::{ActionBufferEdit, ActionBufferEditForward};
use buffer_rebase::{ActionBufferRebase, ActionBufferRebaseForward};
use buffer_rename::{ActionBufferRename, ActionBufferRenameForward};
use buffer_split::{ActionBufferSplit, ActionBufferSplitForward, Split};
use buffer_transform::{ActionBufferTransform, ActionBufferTransformForward};
use buffer_untransform::{ActionBufferUntransform, ActionBufferUntransformForward};
use project_rename::{ActionProjectRename, ActionProjectRenameForward};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Null(null::NullAction),
    ProjectRename(project_rename::ActionProjectRename),
    BufferCreateEmpty(buffer_create_empty::ActionBufferCreateEmpty),
    BufferCreateFromBytes(buffer_create_from_bytes::ActionBufferCreateFromBytes),
    BufferDelete(buffer_delete::ActionBufferDelete),
    BufferTransform(buffer_transform::ActionBufferTransform),
    BufferUntransform(buffer_untransform::ActionBufferUntransform),
    BufferEdit(buffer_edit::ActionBufferEdit),
    BufferCloneShallow(buffer_clone_shallow::ActionBufferCloneShallow),
    BufferClonePartial(buffer_clone_partial::ActionBufferClonePartial),
    BufferRebase(buffer_rebase::ActionBufferRebase),
    BufferRename(buffer_rename::ActionBufferRename),
    BufferSplit(buffer_split::ActionBufferSplit),
}

impl Action {
    pub fn project_rename(name: &str) -> Self {
        Self::ProjectRename(
            ActionProjectRename::new(
                ActionProjectRenameForward {
                    new_name: name.to_string()
                }
            )
        )
    }

    pub fn buffer_create_empty(name: &str, size: usize, base_address: usize) -> Self {
        Self::BufferCreateEmpty(
            ActionBufferCreateEmpty::new(
                ActionBufferCreateEmptyForward {
                    name: name.to_string(),
                    size: size,
                    base_address: base_address
                }
            )
        )
    }

    pub fn buffer_create_from_bytes(name: &str, data: Vec<u8>, base_address: usize) -> Self {
        Self::BufferCreateFromBytes(
            ActionBufferCreateFromBytes::new(
                ActionBufferCreateFromBytesForward {
                    name: name.to_string(),
                    data: data,
                    base_address: base_address
                }
            )
        )
    }

    pub fn buffer_delete(name: &str) -> Self {
        Self::BufferDelete(
            ActionBufferDelete::new(
                ActionBufferDeleteForward {
                    name: name.to_string(),
                }
            )
        )
    }

    pub fn buffer_transform(name: &str, transformation: H2Transformation) -> Self {
        Self::BufferTransform(
            ActionBufferTransform::new(
                ActionBufferTransformForward {
                    name: name.to_string(),
                    transformation: transformation,
                }
            )
        )
    }

    pub fn buffer_untransform(name: &str) -> Self {
        Self::BufferUntransform(
            ActionBufferUntransform::new(
                ActionBufferUntransformForward {
                    name: name.to_string(),
                }
            )
        )
    }

    pub fn buffer_edit(name: &str, new_data: Vec<u8>, offset: usize) -> Self {
        Self::BufferEdit(
            ActionBufferEdit::new(
                ActionBufferEditForward {
                    name: name.to_string(),
                    new_data: new_data,
                    offset: offset,
                }
            )
        )
    }

    pub fn buffer_clone_shallow(clone_from_name: &str, clone_to_name: &str, new_base_address: Option<usize>) -> Self {
        Self::BufferCloneShallow(
            ActionBufferCloneShallow::new(
                ActionBufferCloneShallowForward {
                    clone_from_name: clone_from_name.to_string(),
                    clone_to_name: clone_to_name.to_string(),
                    new_base_address: new_base_address,
                }
            )
        )
    }

    pub fn buffer_clone_partial(clone_from_name: &str, clone_to_name: &str, range: Range<usize>, new_base_address: Option<usize>) -> Self {
        Self::BufferClonePartial(
            ActionBufferClonePartial::new(
                ActionBufferClonePartialForward {
                    clone_from_name: clone_from_name.to_string(),
                    clone_to_name: clone_to_name.to_string(),
                    range: range,
                    new_base_address: new_base_address,
                }
            )
        )
    }

    pub fn buffer_rebase(buffer_name: &str, new_base_address: usize) -> Self {
        Self::BufferRebase(
            ActionBufferRebase::new(
                ActionBufferRebaseForward {
                    buffer_name: buffer_name.to_string(),
                    new_base_address: new_base_address,
                }
            )
        )
    }

    pub fn buffer_rename(original_name: &str, new_name: &str) -> Self {
        Self::BufferRename(
            ActionBufferRename::new(
                ActionBufferRenameForward {
                    original_name: original_name.to_string(),
                    new_name: new_name.to_string(),
                }
            )
        )
    }

    pub fn buffer_split(name: &str, splits: Vec<Split>) -> Self {
        Self::BufferSplit(
            ActionBufferSplit::new(
                ActionBufferSplitForward {
                    name: name.to_string(),
                    splits: splits,
                }
            )
        )
    }
}

impl Command for Action {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        match self {
            Action::Null(a) => a.apply(project),
            Action::ProjectRename(a) => a.apply(project),
            Action::BufferCreateEmpty(a) => a.apply(project),
            Action::BufferCreateFromBytes(a) => a.apply(project),
            Action::BufferDelete(a) => a.apply(project),
            Action::BufferTransform(a) => a.apply(project),
            Action::BufferUntransform(a) => a.apply(project),
            Action::BufferEdit(a) => a.apply(project),
            Action::BufferCloneShallow(a) => a.apply(project),
            Action::BufferClonePartial(a) => a.apply(project),
            Action::BufferRebase(a) => a.apply(project),
            Action::BufferRename(a) => a.apply(project),
            Action::BufferSplit(a) => a.apply(project),
        }
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        match self {
            Action::Null(a) => a.undo(project),
            Action::ProjectRename(a) => a.undo(project),
            Action::BufferCreateEmpty(a) => a.undo(project),
            Action::BufferCreateFromBytes(a) => a.undo(project),
            Action::BufferDelete(a) => a.undo(project),
            Action::BufferTransform(a) => a.undo(project),
            Action::BufferUntransform(a) => a.undo(project),
            Action::BufferEdit(a) => a.undo(project),
            Action::BufferCloneShallow(a) => a.undo(project),
            Action::BufferClonePartial(a) => a.undo(project),
            Action::BufferRebase(a) => a.undo(project),
            Action::BufferRename(a) => a.undo(project),
            Action::BufferSplit(a) => a.undo(project),
        }
    }
}
