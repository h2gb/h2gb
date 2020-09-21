use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::h2project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCloneShallowForward {
    pub clone_from_name: String,
    pub clone_to_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferCloneShallowBackward {
    clone_from_name: String,
    clone_to_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferCloneShallow {
    forward: Option<ActionBufferCloneShallowForward>,
    backward: Option<ActionBufferCloneShallowBackward>,
}

impl ActionBufferCloneShallow {
    pub fn new(forward: ActionBufferCloneShallowForward) -> Self {
        ActionBufferCloneShallow {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, &str)> for ActionBufferCloneShallow {
    fn from(o: (&str, &str)) -> Self {
        ActionBufferCloneShallow {
            forward: Some(ActionBufferCloneShallowForward {
                clone_from_name: o.0.to_string(),
                clone_to_name: o.1.to_string(),
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferCloneShallow {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Make sure we'll be able to insert - this is mostly to save the
        // cloning time
        if project.buffer_exists(&forward.clone_to_name) {
            bail!("Buffer already exists: {}", forward.clone_to_name);
        }

        let original = project.get_buffer(&forward.clone_from_name)?;
        let new_buffer = original.clone_shallow()?;
        project.buffer_insert(&forward.clone_to_name, new_buffer)?;

        // Populate backward for undo
        self.backward = Some(ActionBufferCloneShallowBackward {
            clone_to_name: forward.clone_to_name.clone(),
            clone_from_name: forward.clone_from_name.clone(),
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        // We don't need to keep a copy of the buffer since the name is enough
        // to clone it
        project.buffer_remove(&backward.clone_to_name)?;

        self.forward = Some(ActionBufferCloneShallowForward {
            clone_to_name: backward.clone_to_name.clone(),
            clone_from_name: backward.clone_from_name.clone(),
        });
        self.backward = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use simple_error::SimpleResult;

    use crate::h2project::H2Project;
    use redo::Record;
    use pretty_assertions::assert_eq;
    use crate::action::Action;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer with some data
        record.apply(Action::buffer_create_from_bytes("buffer", b"AAAAAAAAAA".to_vec(), 0x80000000))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("buffer")?.data);
        assert_eq!(0x80000000, record.target().get_buffer("buffer")?.base_address);

        // Clone it
        record.apply(Action::buffer_clone_shallow("buffer", "newbuffer"))?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("newbuffer")?.data);
        assert_eq!(0x80000000, record.target().get_buffer("newbuffer")?.base_address);

        // Edit the original and ensure the change isn't reflected
        record.apply(Action::buffer_edit("buffer", b"BBBBBB".to_vec(), 2))?;
        assert_eq!(b"AABBBBBBAA".to_vec(), record.target().get_buffer("buffer")?.data);
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("newbuffer")?.data);

        // Undo the edit
        record.undo()?;

        // Undo the clone
        record.undo()?;
        assert_eq!(false, record.target().buffer_exists("newbuffer"));
        assert!(record.target().get_buffer("newbuffer").is_err());

        record.redo()?;
        assert_eq!(b"AAAAAAAAAA".to_vec(), record.target().get_buffer("newbuffer")?.data);

        Ok(())
    }

    #[test]
    fn test_action_fails_on_bad_change() -> SimpleResult<()> {
        Ok(())
    }
}
