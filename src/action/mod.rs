use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError};

use h2transformer::H2Transformation;

use crate::h2project::H2Project;

pub mod null;
pub mod project_rename;
pub mod buffer_create_empty;
pub mod buffer_create_from_bytes;
pub mod buffer_delete;
pub mod buffer_transform;
pub mod buffer_untransform;

use project_rename::{ActionProjectRename, ActionProjectRenameForward};
use buffer_create_empty::{ActionBufferCreateEmpty, ActionBufferCreateEmptyForward};
use buffer_create_from_bytes::{ActionBufferCreateFromBytes, ActionBufferCreateFromBytesForward};
use buffer_delete::{ActionBufferDelete, ActionBufferDeleteForward};
use buffer_transform::{ActionBufferTransform, ActionBufferTransformForward};
use buffer_untransform::{ActionBufferUntransform, ActionBufferUntransformForward};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Null(null::NullAction),
    ProjectRename(project_rename::ActionProjectRename),
    BufferCreateEmpty(buffer_create_empty::ActionBufferCreateEmpty),
    BufferCreateFromBytes(buffer_create_from_bytes::ActionBufferCreateFromBytes),
    BufferDelete(buffer_delete::ActionBufferDelete),
    BufferTransform(buffer_transform::ActionBufferTransform),
    BufferUntransform(buffer_untransform::ActionBufferUntransform),
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
        }
    }
}
