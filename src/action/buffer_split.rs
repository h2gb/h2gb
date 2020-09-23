//! Split a buffer into multiple smaller buffers.
//!
//! I expect this will be very common when analyzing files - for example,
//! splitting a PE file into its segments.
//!
//! At the core, this uses a vector of `Split` instances. Each split has a
//! starting address, a name, and optionally a new base address.
//!
//! Importantly, the splitting performed here is contiguous - there can't be
//! empty spots. This means that the split can be two-way - the split-up
//! buffers can be collapsed back to the original buffer for export.

use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};
use std::cmp::Ord;
use std::collections::HashMap;

use crate::h2project::H2Project;
use crate::h2buffer::H2Buffer;

#[derive(Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Split {
    pub start: usize, // Must be first!
    pub new_name: String,
    pub new_base: Option<usize>,
}

// splits are (offset, name, base_address)
#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferSplitForward {
    pub name: String,
    pub splits: Vec<Split>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActionBufferSplitBackward {
    original_name: String,
    original_buffer: H2Buffer,
    splits: Vec<Split>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionBufferSplit {
    forward: Option<ActionBufferSplitForward>,
    backward: Option<ActionBufferSplitBackward>,
}

impl ActionBufferSplit {
    pub fn new(forward: ActionBufferSplitForward) -> Self {
        ActionBufferSplit {
            forward: Some(forward),
            backward: None,
        }
    }
}

impl From<(&str, Vec<Split>)> for ActionBufferSplit {
    fn from(o: (&str, Vec<Split>)) -> Self {
        ActionBufferSplit {
            forward: Some(ActionBufferSplitForward {
                name: o.0.to_string(),
                splits: o.1,
            }),
            backward: None,
        }
    }
}

impl Command for ActionBufferSplit {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward instructions
        let forward = match &self.forward {
            Some(f) => f,
            None => bail!("Failed to apply: missing context"),
        };

        // Get an immutable handle to the buffer for sanity checks
        let buffer = project.get_buffer(&forward.name)?;

        if buffer.is_populated() {
            bail!("Can't split buffer: it's populated");
        }

        // Make sure it's populated
        if forward.splits.len() < 1 {
            bail!("Must have at least one split");
        }

        // Sort the splits - this will sort by the start
        let mut sorted_splits = forward.splits.clone();
        sorted_splits.sort();

        // Sanity check the start
        match sorted_splits.first() {
            Some(first) => {
                if first.start != 0 {
                    bail!("First split must start at 0");
                }
            },
            None => bail!("Must have at least one split"),
        };

        // Create the buffers - this will allow us to validate before we start
        // changing things around
        let mut buffers: HashMap<String, H2Buffer> = HashMap::new();

        for (index, split) in sorted_splits.iter().enumerate() {
            // Check if such a buffer exists
            if project.buffer_exists(&split.new_name) {
                bail!("Buffer already exists in project: {}", split.new_name);
            }

            // Check for duplicate name
            if buffers.contains_key(&split.new_name) {
                bail!("Duplicate name: {}", split.new_name);
            }

            // Check the length
            let start = split.start;
            let end = match sorted_splits.get(index + 1) {
                Some(s) => s.start,
                None => buffer.len(),
            };

            if end > buffer.len() {
                bail!("Split is too large");
            }

            // Create the new buffer
            let data = Vec::from(&buffer.data[start..end]); // The subset of data
            let base_address =  match split.new_base {
                Some(b) => b,
                None => buffer.base_address + start,
            };

            buffers.insert(split.new_name.clone(), H2Buffer::new(data, base_address)?);
        }

        // This will insert all or nothing so we don't end up in a half-way state
        project.buffer_insert_multiple(buffers)?;

        // This shouldn't fail, but if it does, we need to do our best to back
        // out the change.
        let original_buffer = match project.buffer_remove(&forward.name) {
            Ok(b) => b,
            Err(e) => {
                // Try and fix what we've done
                for split in &forward.splits {
                    // Ignore errors here
                    project.buffer_remove(&split.new_name).ok();
                }

                return Err(e);
            },
        };

        // Populate backward for undo
        self.backward = Some(ActionBufferSplitBackward {
            original_name: forward.name.clone(),
            original_buffer: original_buffer,
            splits: sorted_splits,
        });
        self.forward = None;

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        let backward = match &self.backward {
            Some(b) => b,
            None => bail!("Failed to undo: missing context"),
        };

        // Remove the split buffers
        for split in &backward.splits {
            project.buffer_remove(&split.new_name)?;
        }

        // Restore the original buffer
        // We have to clone to avoid putting things in a potentially bad state
        project.buffer_insert(&backward.original_name, backward.original_buffer.clone_shallow(None)?)?;

        // Save the action back to forward
        self.forward = Some(ActionBufferSplitForward {
            name: backward.original_name.clone(),
            splits: backward.splits.clone(),
        });
        self.backward = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use redo::Record;
    use simple_error::SimpleResult;

    use crate::action::Action;
    use crate::h2project::H2Project;

    #[test]
    fn test_action() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        // Split it
        record.apply(Action::buffer_split("buffer", vec![
            Split { new_name: "e".to_string(), start: 4,  new_base: None },
            Split { new_name: "f".to_string(), start: 5,  new_base: None },
            Split { new_name: "g".to_string(), start: 6,  new_base: None },
            Split { new_name: "p".to_string(), start: 15, new_base: Some(1234) },

            // Put the first one last to ensure it sorts
            Split { new_name: "z".to_string(), start: 0,  new_base: None },
        ]))?;

        assert_eq!(false, record.target().buffer_exists("buffer"));

        assert_eq!(true, record.target().buffer_exists("z"));
        assert_eq!(b"ABCD".to_vec(), record.target().get_buffer("z")?.data);
        assert_eq!(0x80000000, record.target().get_buffer("z")?.base_address);

        assert_eq!(true, record.target().buffer_exists("e"));
        assert_eq!(b"E".to_vec(), record.target().get_buffer("e")?.data);
        assert_eq!(0x80000004, record.target().get_buffer("e")?.base_address);

        assert_eq!(true, record.target().buffer_exists("f"));
        assert_eq!(b"F".to_vec(), record.target().get_buffer("f")?.data);
        assert_eq!(0x80000005, record.target().get_buffer("f")?.base_address);

        assert_eq!(true, record.target().buffer_exists("g"));
        assert_eq!(b"GHIJKLMNO".to_vec(), record.target().get_buffer("g")?.data);
        assert_eq!(0x80000006, record.target().get_buffer("g")?.base_address);

        assert_eq!(true, record.target().buffer_exists("p"));
        assert_eq!(b"P".to_vec(), record.target().get_buffer("p")?.data);
        // This base_address was customized in the split
        assert_eq!(1234, record.target().get_buffer("p")?.base_address);

        // Undo
        record.undo()?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        assert_eq!(false, record.target().buffer_exists("z"));
        assert_eq!(false, record.target().buffer_exists("e"));
        assert_eq!(false, record.target().buffer_exists("f"));
        assert_eq!(false, record.target().buffer_exists("g"));
        assert_eq!(false, record.target().buffer_exists("p"));

        // Redo
        record.redo()?;
        assert_eq!(false, record.target().buffer_exists("buffer"));

        assert_eq!(true, record.target().buffer_exists("z"));
        assert_eq!(b"ABCD".to_vec(), record.target().get_buffer("z")?.data);
        assert_eq!(0x80000000, record.target().get_buffer("z")?.base_address);

        assert_eq!(true, record.target().buffer_exists("e"));
        assert_eq!(b"E".to_vec(), record.target().get_buffer("e")?.data);
        assert_eq!(0x80000004, record.target().get_buffer("e")?.base_address);

        assert_eq!(true, record.target().buffer_exists("f"));
        assert_eq!(b"F".to_vec(), record.target().get_buffer("f")?.data);
        assert_eq!(0x80000005, record.target().get_buffer("f")?.base_address);

        assert_eq!(true, record.target().buffer_exists("g"));
        assert_eq!(b"GHIJKLMNO".to_vec(), record.target().get_buffer("g")?.data);
        assert_eq!(0x80000006, record.target().get_buffer("g")?.base_address);

        assert_eq!(true, record.target().buffer_exists("p"));
        assert_eq!(b"P".to_vec(), record.target().get_buffer("p")?.data);
        // This base_address was customized in the split
        assert_eq!(1234, record.target().get_buffer("p")?.base_address);

        Ok(())
    }

    #[test]
    fn test_fails_when_no_zero() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        // Split it
        assert!(record.apply(Action::buffer_split("buffer", vec![
            Split { new_name: "e".to_string(), start: 4,  new_base: None },
            Split { new_name: "f".to_string(), start: 8,  new_base: None },
        ])).is_err());
        assert_eq!(false, record.target().buffer_exists("e"));
        assert_eq!(false, record.target().buffer_exists("f"));

        Ok(())
    }

    #[test]
    fn test_fails_when_no_splits() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        // Split it
        assert!(record.apply(Action::buffer_split("buffer", vec![])).is_err());

        Ok(())
    }

    #[test]
    fn test_fails_when_too_long() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        // Split it
        assert!(record.apply(Action::buffer_split("buffer", vec![
            Split { new_name: "a".to_string(), start: 0,    new_base: None },
            Split { new_name: "e".to_string(), start: 100,  new_base: None },
        ])).is_err());
        assert_eq!(false, record.target().buffer_exists("a"));
        assert_eq!(false, record.target().buffer_exists("e"));

        Ok(())
    }

    #[test]
    fn test_fails_when_zero_sized_area() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        // Split it
        assert!(record.apply(Action::buffer_split("buffer", vec![
            Split { new_name: "a".to_string(), start: 0,  new_base: None },
            Split { new_name: "e".to_string(), start: 4,  new_base: None },
            Split { new_name: "f".to_string(), start: 0,  new_base: None },
        ])).is_err());
        assert_eq!(false, record.target().buffer_exists("a"));
        assert_eq!(false, record.target().buffer_exists("e"));
        assert_eq!(false, record.target().buffer_exists("f"));

        Ok(())
    }

    #[test]
    fn test_fails_when_buffer_doesnt_exist() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Split it
        assert!(record.apply(Action::buffer_split("nosuchbuffer", vec![
            Split { new_name: "a".to_string(), start: 0,  new_base: None },
            Split { new_name: "e".to_string(), start: 4,  new_base: None },
        ])).is_err());
        assert_eq!(false, record.target().buffer_exists("a"));
        assert_eq!(false, record.target().buffer_exists("e"));

        Ok(())
    }

    #[test]
    fn test_fails_when_duplicates() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        // Split it
        assert!(record.apply(Action::buffer_split("buffer", vec![
            Split { new_name: "a".to_string(), start: 0,  new_base: None },
            Split { new_name: "a".to_string(), start: 4,  new_base: None },
        ])).is_err());
        assert_eq!(false, record.target().buffer_exists("a"));

        Ok(())
    }

    #[test]
    fn test_fails_when_buffer_exists() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer
        record.apply(Action::buffer_create_from_bytes("buffer", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        record.apply(Action::buffer_create_from_bytes("exists", b"ABCDEFGHIJKLMNOP".to_vec(), 0x80000000))?;
        assert_eq!(true, record.target().buffer_exists("buffer"));

        // Split it
        assert!(record.apply(Action::buffer_split("buffer", vec![
            Split { new_name: "a".to_string(), start: 0,  new_base: None },
            Split { new_name: "exists".to_string(), start: 4,  new_base: None },
        ])).is_err());
        assert_eq!(false, record.target().buffer_exists("a"));

        Ok(())
    }
}
