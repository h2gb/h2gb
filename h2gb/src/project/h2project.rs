//! Implement the core project, which everything else builds off of.
//!
//! This is likely to change a lot as I get the foundations built, so it's not
//! quite ready for detailed comments just yet. :)

use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult, SimpleError};
use std::collections::HashMap;
use std::fmt;

use crate::project::H2Buffer;

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
    pub fn new(name: impl AsRef<str>, version: impl AsRef<str>) -> Self {
        H2Project {
            name: name.as_ref().to_string(),
            version: version.as_ref().to_string(),

            buffers: HashMap::new(),
        }
    }

    // fn multi_key(buffer: &str, layer: &str) -> (String, String) {
    //     (buffer.to_string(), layer.to_string())
    // }

    pub fn buffers(&self) -> &HashMap<String, H2Buffer> {
        return &self.buffers;
    }

    pub fn buffer_exists(&self, buffer: impl AsRef<str>) -> bool {
        self.buffers.contains_key(buffer.as_ref())
    }

    pub fn buffer_insert(&mut self, name: impl AsRef<str>, buffer: H2Buffer) -> SimpleResult<()> {
        // Sanity check
        if name.as_ref() == "" {
            bail!("Buffer must have a name");
        }

        if self.buffer_exists(name.as_ref()) {
            bail!("Buffer already exists: {}", name.as_ref());
        }

        // Go
        // TODO: Check and insert at the same time
        self.buffers.insert(name.as_ref().to_string(), buffer);

        Ok(())
    }

    // Note: In the future, we should check for references to this buffer to
    // ensure we aren't breaking anything else
    pub fn buffer_can_be_removed(&self, buffer: impl AsRef<str>) -> SimpleResult<bool> {
        match self.buffer_get(buffer.as_ref()) {
            Some(buffer) => Ok(!buffer.is_populated()),
            None => bail!("No such buffer: {}", buffer.as_ref()),
        }
    }

    pub fn buffer_remove(&mut self, buffer: impl AsRef<str>) -> SimpleResult<H2Buffer> {
        // Sanity check
        if !self.buffer_can_be_removed(&buffer)? {
            bail!("Cannot remove buffer: {}", buffer.as_ref());
        }

        // Go
        match self.buffers.remove(buffer.as_ref()) {
            Some(b) => Ok(b),
            None => bail!("Buffer not found"),
        }
    }

    pub fn buffer_get(&self, buffer: impl AsRef<str>) -> Option<&H2Buffer> {
        self.buffers.get(buffer.as_ref())
    }

    pub fn buffer_get_or_err(&self, buffer: impl AsRef<str>) -> SimpleResult<&H2Buffer> {
        self.buffer_get(&buffer).ok_or(
            SimpleError::new(format!("Could not find buffer {}", buffer.as_ref()))
        )
    }

    pub fn buffer_get_mut(&mut self, buffer: impl AsRef<str>) -> Option<&mut H2Buffer> {
        self.buffers.get_mut(buffer.as_ref())
    }

    pub fn buffer_get_mut_or_err(&mut self, buffer: impl AsRef<str>) -> SimpleResult<&mut H2Buffer> {
        self.buffer_get_mut(&buffer).ok_or(
            SimpleError::new(format!("Could not find buffer {}", buffer.as_ref()))
        )
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

    // pub fn buffer_rename(&mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> SimpleResult<()> {
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

    // Remove an entry, and any others that were inserted along with it
    // pub fn entry_remove(&mut self, buffer: impl AsRef<str>, layer: impl AsRef<str>, offset: usize) -> SimpleResult<Vec<(String, String, Option<H2Type>, usize)>> {
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
    //     buffers.insert("buffer1", H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?);
    //     buffers.insert("buffer2", H2Buffer::new("name", b"EFGH".to_vec(), 0x100)?);
    //     buffers.insert("buffer3", H2Buffer::new("name", b"IJKL".to_vec(), 0x100)?);

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
    //     buffers.insert("buffer1", H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?);
    //     buffers.insert("buffer2", H2Buffer::new("name", b"EFGH".to_vec(), 0x100)?);
    //     buffers.insert("buffer3", H2Buffer::new("name", b"IJKL".to_vec(), 0x100)?);
    //     buffers.insert("duplicate", H2Buffer::new("name", b"YYYY".to_vec(), 0x100)?);

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
