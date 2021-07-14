//! Implement the core project, which everything else builds off of.
//!
//! This is likely to change a lot as I get the foundations built, so it's not
//! quite ready for detailed comments just yet. :)

use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::collections::HashMap;
use std::fmt;

use crate::datatype::H2Type;
use crate::project::{H2Buffer, H2Layer, H2Entry};

// H2Project is the very core, and the root of undo. All actions will be taken
// via this object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Project {
    pub name: String,
    pub version: String,

    // Buffers that exist, indexed by their name; layers are stored in their
    // respective buffer
    buffers: HashMap<String, H2Buffer>,
}

impl H2Project {
    pub fn new(name: &str, version: &str) -> Self {
        H2Project {
            name: String::from(name),
            version: String::from(version),

            buffers: HashMap::new(),
        }
    }

    fn multi_key(buffer: &str, layer: &str) -> (String, String) {
        (buffer.to_string(), layer.to_string())
    }

    pub fn buffers(&self) -> &HashMap<String, H2Buffer> {
        return &self.buffers;
    }

    // TODO: Should this return an Option?
    pub fn buffer_get(&self, buffer: &str) -> Option<&H2Buffer> {
        self.buffers.get(buffer)
    }

    pub fn buffer_get_mut(&mut self, buffer: &str) -> Option<&mut H2Buffer> {
        self.buffers.get_mut(buffer)
    }

    pub fn buffer_exists(&self, buffer: &str) -> bool {
        self.buffers.contains_key(buffer)
    }

    pub fn buffer_insert(&mut self, name: &str, buffer: H2Buffer) -> SimpleResult<()> {
        // Sanity check
        if name == "" {
            bail!("Buffer must have a name");
        }

        if self.buffer_exists(name) {
            bail!("Buffer already exists: {}", name);
        }

        // Go
        // TODO: Check and insert at the same time
        self.buffers.insert(name.to_string(), buffer);

        Ok(())
    }

    // Guarantees either all or none are inserted
    // pub fn buffer_insert_multiple(&mut self, mut buffers: HashMap<String, H2Buffer>) -> SimpleResult<()> {
    //     // Validate first
    //     for name in buffers.keys() {
    //         if self.buffer_exists(name) {
    //             bail!("Buffer already exists: {}", name);
    //         }
    //     }

    //     // Then insert
    //     for (name, buffer) in buffers.drain() {
    //         self.buffers.insert(name, buffer);
    //     }

    //     Ok(())
    // }

    // Note: In the future, we should check for references to this buffer to
    // ensure we aren't breaking anything else
    pub fn buffer_can_be_removed(&self, buffer: &str) -> SimpleResult<bool> {
        match self.buffer_get(buffer) {
            Some(buffer) => Ok(!buffer.is_populated()),
            None => bail!("No such buffer: {}", buffer),
        }
    }

    pub fn buffer_remove(&mut self, buffer: &str) -> SimpleResult<H2Buffer> {
        // Sanity check
        if !self.buffer_can_be_removed(buffer)? {
            bail!("Cannot remove buffer: {}", buffer);
        }

        // Go
        match self.buffers.remove(buffer) {
            Some(b) => Ok(b),
            None => bail!("Buffer not found"),
        }
    }

    // pub fn buffer_rename(&mut self, from: &str, to: &str) -> SimpleResult<()> {
    //     let buffer = self.buffer_get(from)?;

    //     // Sanity check
    //     // (We'll probably want to allow renaming populated buffers eventually)
    //     if buffer.is_populated() {
    //         bail!("Buffer has data in it");
    //     }

    //     // Ensure a name was given
    //     if to == "" {
    //         bail!("Can't rename to a blank name");
    //     }

    //     // Ensure the new name doesn't exist
    //     if self.buffer_exists(to) {
    //         bail!("Target buffer name already exists");
    //     }

    //     // Remove the old name
    //     let b = match self.buffers.remove(from) {
    //         Some(b) => b,
    //         None => bail!("Buffer not found"),
    //     };

    //     // Add the new name
    //     self.buffers.insert(to.to_string(), b);

    //     Ok(())
    // }

    pub fn layer_add(&mut self, buffer: &str, layer: &str) -> SimpleResult<()> {
        // Get the size (from a read-only version of the buffer)
        match self.buffer_get_mut(buffer) {
            Some(buffer) => buffer.layer_add(layer)?,
            None => bail!("Can't add layer: no such buffer {}", buffer),
        };

        Ok(())
    }

    pub fn layer_remove(&mut self, buffer: &str, layer: &str) -> SimpleResult<()> {
        match self.buffer_get_mut(buffer) {
            Some(b) => b.layer_remove(layer)?,
            None => bail!("Can't remove layer: no such buffer '{}'", buffer),
        };

        Ok(())
    }

    pub fn layer_exists(&self, buffer: &str, layer: &str) -> bool {
        match self.buffer_get(buffer) {
            Some(b) => b.layer_exists(layer),
            None => false,
        }
    }

    pub fn layer_get(&self, buffer: &str, layer: &str) -> Option<&H2Layer> {
        self.buffer_get(buffer)?.layer_get(layer)
    }

    pub fn layer_get_mut(&mut self, buffer: &str, layer: &str) -> Option<&mut H2Layer> {
        self.buffer_get_mut(buffer)?.layer_get_mut(layer)
    }

    // Does not validate whether it could actually be inserted
    pub fn entry_create(&self, buffer: &str, abstract_type: H2Type, offset: usize) -> SimpleResult<H2Entry> {
        let buffer = match self.buffer_get(buffer) {
            Some(b) => b,
            None => bail!("Couldn't find buffer {}", buffer),
        };

        buffer.entry_create(abstract_type, offset)
    }

    pub fn entry_insert(&mut self, buffer: &str, layer: &str, entry: H2Entry) -> SimpleResult<()> {
        let buffer = match self.buffer_get_mut(buffer) {
            Some(b) => b,
            None => bail!("Couldn't find buffer {}", buffer),
        };

        buffer.entry_insert(layer, entry)
    }

    pub fn entry_create_and_insert(&mut self, buffer: &str, layer: &str, abstract_type: H2Type, offset: usize) -> SimpleResult<()> {
        let entry = self.entry_create(buffer, abstract_type, offset)?;

        self.entry_insert(buffer, layer, entry)
    }

    pub fn entry_get(&self, buffer: &str, layer: &str, offset: usize) -> Option<&H2Entry> {
        self.layer_get(buffer, layer)?.entry_get(offset)
    }

    pub fn entry_get_mut(&mut self, buffer: &str, layer: &str, offset: usize) -> Option<&mut H2Entry> {
        self.layer_get_mut(buffer, layer)?.entry_get_mut(offset)
    }

    pub fn entry_remove(&mut self, buffer: &str, layer: &str, offset: usize) -> Option<H2Entry> {
        self.buffer_get_mut(buffer)?.entry_remove(layer, offset)
    }

    pub fn comment_set(&mut self, buffer: &str, layer: &str, offset: usize, comment: Option<String>) -> SimpleResult<Option<String>> {
        let buffer = match self.buffer_get_mut(buffer) {
            Some(l) => l,
            None => bail!("Couldn't find buffer {} to add comment", buffer),
        };

        buffer.comment_set(layer, offset, comment)
    }

    // Remove an entry, and any others that were inserted along with it
    // pub fn entry_remove(&mut self, buffer: &str, layer: &str, offset: usize) -> SimpleResult<Vec<(String, String, Option<H2Type>, usize)>> {
    //     let multi_key = Self::multi_key(buffer, layer);
    //     let entries = self.entries.remove_entries(&multi_key, offset)?;

    //     let entries: Vec<_> = entries.iter().filter_map(|e| {
    //         match e {
    //             Some(entry) => {
    //                 // Get the offset from the BumpyEntry
    //                 let offset = entry.range.start;

    //                 // Get the vector from the MultiEntry
    //                 let buffer_layer = entry.entry.vector.clone();

    //                 // Get the H2Type from the data
    //                 let datatype = entry.entry.data.creator();

    //                 // Put them together in the same order as entry_create_from_type()'s arguments
    //                 Some((buffer_layer.0, buffer_layer.1, datatype, offset))
    //             },
    //             None => None,
    //         }
    //     }).collect();

    //     Ok(entries)
    // }
}

impl fmt::Display for H2Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}, version: {}", self.name, self.version)?;

        for (_name, buffer) in self.buffers() {
            writeln!(f, "{}", buffer)?;
            writeln!(f, "")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_buffer_insert() -> SimpleResult<()> {
        let mut project = H2Project::new("name", "1.0");
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;

        // No buffer, can't be retrieved
        assert_eq!(false, project.buffer_exists("buffer"));
        assert!(project.buffer_get("buffer").is_none());

        // Insert it
        project.buffer_insert("buffer", buffer)?;

        // Now it exists and can be retrieved
        assert_eq!(true, project.buffer_exists("buffer"));
        assert_eq!(b"ABCD".to_vec(), project.buffer_get("buffer").unwrap().data);
        assert_eq!(0x100, project.buffer_get("buffer").unwrap().base_address);

        // Try to insert a duplicate, and fail
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;
        assert!(project.buffer_insert("buffer", buffer).is_err());

        // Try to insert a blank name, and fail
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;
        assert!(project.buffer_insert("", buffer).is_err());

        Ok(())
    }

    // #[test]
    // fn test_buffer_insert_multiple() -> SimpleResult<()> {
    //     let mut project = H2Project::new("name", "1.0");

    //     let mut buffers: HashMap<String, H2Buffer> = HashMap::new();
    //     buffers.insert("buffer1".to_string(), H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?);
    //     buffers.insert("buffer2".to_string(), H2Buffer::new("name", b"EFGH".to_vec(), 0x100)?);
    //     buffers.insert("buffer3".to_string(), H2Buffer::new("name", b"IJKL".to_vec(), 0x100)?);

    //     assert_eq!(false, project.buffer_exists("buffer1"));
    //     assert_eq!(false, project.buffer_exists("buffer2"));
    //     assert_eq!(false, project.buffer_exists("buffer3"));

    //     project.buffer_insert_multiple(buffers)?;

    //     assert_eq!(true, project.buffer_exists("buffer1"));
    //     assert_eq!(true, project.buffer_exists("buffer2"));
    //     assert_eq!(true, project.buffer_exists("buffer3"));

    //     Ok(())
    // }

    // fn test_buffer_insert_multiple_fails_clean() -> SimpleResult<()> {
    //     let mut project = H2Project::new("name", "1.0");

    //     project.buffer_insert("duplicate", H2Buffer::new("name", b"ZZZZ".to_vec(), 0x200)?)?;

    //     let mut buffers: HashMap<String, H2Buffer> = HashMap::new();
    //     buffers.insert("buffer1".to_string(), H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?);
    //     buffers.insert("buffer2".to_string(), H2Buffer::new("name", b"EFGH".to_vec(), 0x100)?);
    //     buffers.insert("buffer3".to_string(), H2Buffer::new("name", b"IJKL".to_vec(), 0x100)?);
    //     buffers.insert("duplicate".to_string(), H2Buffer::new("name", b"YYYY".to_vec(), 0x100)?);

    //     assert_eq!(false, project.buffer_exists("buffer1"));
    //     assert_eq!(false, project.buffer_exists("buffer2"));
    //     assert_eq!(false, project.buffer_exists("buffer3"));
    //     assert_eq!(true, project.buffer_exists("duplicate"));

    //     project.buffer_insert_multiple(buffers)?;

    //     // None of them should be inserted
    //     assert_eq!(false, project.buffer_exists("buffer1"));
    //     assert_eq!(false, project.buffer_exists("buffer2"));
    //     assert_eq!(false, project.buffer_exists("buffer3"));
    //     assert_eq!(true, project.buffer_exists("duplicate"));

    //     Ok(())
    // }

    #[test]
    fn test_buffer_remove() -> SimpleResult<()> {
        let mut project = H2Project::new("name", "1.0");
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;

        // Insert it
        project.buffer_insert("buffer", buffer)?;

        // It's there
        assert_eq!(true, project.buffer_exists("buffer"));

        // Remove it
        let buffer = project.buffer_remove("buffer")?;

        // It's gone
        assert_eq!(false, project.buffer_exists("buffer"));

        // Make sure it's the one we inserted
        assert_eq!(buffer.data, b"ABCD".to_vec());
        assert_eq!(buffer.base_address, 0x100);

        Ok(())
    }

    #[test]
    fn test_buffer_remove_no_such_buffer() -> SimpleResult<()> {
        let mut project = H2Project::new("name", "1.0");

        // Remove a fake buffer
        assert!(project.buffer_remove("fakebuffer").is_err());

        Ok(())
    }

    // #[test]
    // fn test_buffer_rename() -> SimpleResult<()> {
    //     let mut project = H2Project::new("name", "1.0");
    //     let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;

    //     // Insert it
    //     project.buffer_insert("buffer", buffer)?;

    //     // It's there
    //     assert_eq!(true, project.buffer_exists("buffer"));

    //     // Rename it
    //     project.buffer_rename("buffer", "newbuffer")?;

    //     // It's gone from the old name, but exists on the new name
    //     assert_eq!(false, project.buffer_exists("buffer"));
    //     assert_eq!(true, project.buffer_exists("newbuffer"));

    //     Ok(())
    // }

    // #[test]
    // fn test_buffer_rename_no_such_buffer() -> SimpleResult<()> {
    //     let mut project = H2Project::new("name", "1.0");

    //     // Rename a fake buffer
    //     assert!(project.buffer_rename("fakebuffer", "newbuffer").is_err());

    //     Ok(())
    // }
}
