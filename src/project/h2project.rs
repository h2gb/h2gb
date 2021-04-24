//! Implement the core project, which everything else builds off of.
//!
//! This is likely to change a lot as I get the foundations built, so it's not
//! quite ready for detailed comments just yet. :)

use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::collections::HashMap;
use std::fmt;

use crate::datatype::{H2Type, Offset, ResolvedType};
use crate::sized_number::Context;
use crate::multi_vector::MultiVector;

use crate::project::h2buffer::H2Buffer;
use crate::project::h2entry::H2Entry;

// H2Project is the very core, and the root of undo. All actions will be taken
// via this object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Project {
    pub name: String,
    pub version: String,

    // Buffers that exist, indexed by their name; layers are stored in their
    // respective buffer
    buffers: HashMap<String, H2Buffer>,

    // Entries that exist, index by buffer + layer; additionally, H2Entry
    // tracks its index and size within the buffer + layer
    entries: MultiVector<(String, String), H2Entry>,
}

impl H2Project {
    pub fn new(name: &str, version: &str) -> Self {
        H2Project {
            name: String::from(name),
            version: String::from(version),

            buffers: HashMap::new(),

            // Entries span across buffers and layers, referencing each other;
            // hence, the only logical thing I can think of is to store them
            // separately in a data structure
            entries: MultiVector::new(),
        }
    }

    fn multi_key(buffer: &str, layer: &str) -> (String, String) {
        (buffer.to_string(), layer.to_string())
    }

    pub fn buffers(&self) -> &HashMap<String, H2Buffer> {
        return &self.buffers;
    }

    // TODO: Should this return an Option?
    pub fn get_buffer(&self, name: &str) -> SimpleResult<&H2Buffer> {
        match self.buffers.get(name) {
            Some(b) => Ok(b),
            None => bail!("Buffer {} not found", name),
        }
    }

    pub fn get_buffer_mut(&mut self, name: &str) -> SimpleResult<&mut H2Buffer> {
        match self.buffers.get_mut(name) {
            Some(b) => Ok(b),
            None => bail!("Buffer {} not found", name),
        }
    }

    pub fn buffer_exists(&self, name: &str) -> bool {
        self.buffers.contains_key(name)
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
        self.buffers.insert(name.to_string(), buffer);

        Ok(())
    }

    // Guarantees either all or none are inserted
    pub fn buffer_insert_multiple(&mut self, mut buffers: HashMap<String, H2Buffer>) -> SimpleResult<()> {
        // Validate first
        for name in buffers.keys() {
            if self.buffer_exists(name) {
                bail!("Buffer already exists: {}", name);
            }
        }

        // Then insert
        for (name, buffer) in buffers.drain() {
            self.buffers.insert(name, buffer);
        }

        Ok(())
    }

    // Note: In the future, we should check for references to this buffer to
    // ensure we aren't breaking anything else
    pub fn buffer_can_be_removed(&self, name: &str) -> SimpleResult<()> {
        let buffer = self.get_buffer(name)?;

        if buffer.is_populated() {
            bail!("Buffer has data in it");
        }

        Ok(())
    }

    pub fn buffer_remove(&mut self, name: &str) -> SimpleResult<H2Buffer> {
        // Sanity check
        self.buffer_can_be_removed(name)?;

        // Go
        match self.buffers.remove(name) {
            Some(b) => Ok(b),
            None => bail!("Buffer not found"),
        }
    }

    pub fn buffer_rename(&mut self, from: &str, to: &str) -> SimpleResult<()> {
        let buffer = self.get_buffer(from)?;

        // Sanity check
        // (We'll probably want to allow renaming populated buffers eventually)
        if buffer.is_populated() {
            bail!("Buffer has data in it");
        }

        // Ensure a name was given
        if to == "" {
            bail!("Can't rename to a blank name");
        }

        // Ensure the new name doesn't exist
        if self.buffer_exists(to) {
            bail!("Target buffer name already exists");
        }

        // Remove the old name
        let b = match self.buffers.remove(from) {
            Some(b) => b,
            None => bail!("Buffer not found"),
        };

        // Add the new name
        self.buffers.insert(to.to_string(), b);

        Ok(())
    }

    pub fn layer_add(&mut self, buffer: &str, layer: &str) -> SimpleResult<()> {
        let multi_key = Self::multi_key(buffer, layer);

        // Get the size (from a read-only version of the buffer)
        let actual_buffer = self.get_buffer(buffer)?;
        let buffer_size = actual_buffer.len();

        // Allocate room in the multi vector
        self.entries.create_vector(multi_key.clone(), buffer_size)?;

        // Get a read/write handle to the buffer, and add the field
        let actual_buffer = self.get_buffer_mut(buffer)?;
        match actual_buffer.layer_add(layer) {
            Ok(_) => (),
            Err(e) => {
                // Try to roll back
                self.entries.destroy_vector(&multi_key)?;
                bail!("Failed to add layer: {}", e);
            },
        };

        Ok(())
    }

    pub fn layer_remove(&mut self, buffer: &str, layer: &str) -> SimpleResult<()> {
        let multi_key = Self::multi_key(buffer, layer);

        // Delete from the MultiVector (this will fail if the buffer isn't empty)
        let size = self.entries.destroy_vector(&multi_key)?;

        // Get the vector, and roll back if we need to
        // If this fails (which should never happen), roll back the destroy from earlier
        let actual_buffer = match self.get_buffer_mut(buffer) {
            Ok(b) => b,
            Err(e) => {
                // If this fails, we're kinda outta luck unfortunately
                self.entries.create_vector(multi_key, size)?;
                bail!("Couldn't remove layer: {}", e);
            }
        };

        // Remove the layer from the buffer
        // If this fails (which should never happen), roll back the destroy from earlier
        match actual_buffer.layer_remove(layer) {
            Ok(()) => (),
            Err(e) => {
                // If this fails, we're kinda outta luck unfortunately
                self.entries.create_vector(multi_key, size)?;
                bail!("Couldn't remove layer: {}", e);
            }
        };

        Ok(())
    }

    pub fn layer_exists(&self, buffer: &str, layer: &str) -> SimpleResult<bool> {
        if !self.buffer_exists(buffer) {
            bail!("No such buffer: {}", buffer);
        }

        Ok(self.entries.vector_exists(&Self::multi_key(buffer, layer)))
    }

    pub fn entry_create_from_type(&mut self, buffer: &str, layer: &str, datatype: H2Type, start: usize) -> SimpleResult<ResolvedType> {
        // Ensure that the buffer and layer exist
        let actual_buffer = self.get_buffer(buffer)?;
        let multi_key = Self::multi_key(buffer, layer);

        // Get a pointer to the data
        let data = &actual_buffer.data;

        // Resolve the data
        let offset = Offset::Dynamic(Context::new(data).at(start as u64)); // TODO: I don't like this cast
        let resolved = datatype.resolve(offset, None)?;

        // Create the entry object
        let entry = H2Entry::new(resolved.clone(), Some(datatype));

        // Insert it
        // This will fail if there's already something there
        self.entries.insert_entry(&multi_key, entry)?;

        Ok(resolved)
    }

    pub fn entry_get(&self, buffer: &str, layer: &str, start: usize) -> SimpleResult<&H2Entry> {
        let entry = match self.entries.get_entry(&(buffer.to_string(), layer.to_string()), start) {
            Some(entry) => entry,
            None        => bail!("Could not find entry"),
        };

        Ok(&entry.entry.data)
    }
}

impl fmt::Display for H2Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}, version: {}\n", self.name, self.version)?;

        for entry in self.entries.into_iter() {
            write!(f, "{}\n", entry.entry.data)?;
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
        assert!(project.get_buffer("buffer").is_err());

        // Insert it
        project.buffer_insert("buffer", buffer)?;

        // Now it exists and can be retrieved
        assert_eq!(true, project.buffer_exists("buffer"));
        assert_eq!(b"ABCD".to_vec(), project.get_buffer("buffer")?.data);
        assert_eq!(0x100, project.get_buffer("buffer")?.base_address);

        // Try to insert a duplicate, and fail
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;
        assert!(project.buffer_insert("buffer", buffer).is_err());

        // Try to insert a blank name, and fail
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;
        assert!(project.buffer_insert("", buffer).is_err());

        Ok(())
    }

    #[test]
    fn test_buffer_insert_multiple() -> SimpleResult<()> {
        let mut project = H2Project::new("name", "1.0");

        let mut buffers: HashMap<String, H2Buffer> = HashMap::new();
        buffers.insert("buffer1".to_string(), H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?);
        buffers.insert("buffer2".to_string(), H2Buffer::new("name", b"EFGH".to_vec(), 0x100)?);
        buffers.insert("buffer3".to_string(), H2Buffer::new("name", b"IJKL".to_vec(), 0x100)?);

        assert_eq!(false, project.buffer_exists("buffer1"));
        assert_eq!(false, project.buffer_exists("buffer2"));
        assert_eq!(false, project.buffer_exists("buffer3"));

        project.buffer_insert_multiple(buffers)?;

        assert_eq!(true, project.buffer_exists("buffer1"));
        assert_eq!(true, project.buffer_exists("buffer2"));
        assert_eq!(true, project.buffer_exists("buffer3"));

        Ok(())
    }

    fn test_buffer_insert_multiple_fails_clean() -> SimpleResult<()> {
        let mut project = H2Project::new("name", "1.0");

        project.buffer_insert("duplicate", H2Buffer::new("name", b"ZZZZ".to_vec(), 0x200)?)?;

        let mut buffers: HashMap<String, H2Buffer> = HashMap::new();
        buffers.insert("buffer1".to_string(), H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?);
        buffers.insert("buffer2".to_string(), H2Buffer::new("name", b"EFGH".to_vec(), 0x100)?);
        buffers.insert("buffer3".to_string(), H2Buffer::new("name", b"IJKL".to_vec(), 0x100)?);
        buffers.insert("duplicate".to_string(), H2Buffer::new("name", b"YYYY".to_vec(), 0x100)?);

        assert_eq!(false, project.buffer_exists("buffer1"));
        assert_eq!(false, project.buffer_exists("buffer2"));
        assert_eq!(false, project.buffer_exists("buffer3"));
        assert_eq!(true, project.buffer_exists("duplicate"));

        project.buffer_insert_multiple(buffers)?;

        // None of them should be inserted
        assert_eq!(false, project.buffer_exists("buffer1"));
        assert_eq!(false, project.buffer_exists("buffer2"));
        assert_eq!(false, project.buffer_exists("buffer3"));
        assert_eq!(true, project.buffer_exists("duplicate"));

        Ok(())
    }

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

    #[test]
    fn test_buffer_rename() -> SimpleResult<()> {
        let mut project = H2Project::new("name", "1.0");
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x100)?;

        // Insert it
        project.buffer_insert("buffer", buffer)?;

        // It's there
        assert_eq!(true, project.buffer_exists("buffer"));

        // Rename it
        project.buffer_rename("buffer", "newbuffer")?;

        // It's gone from the old name, but exists on the new name
        assert_eq!(false, project.buffer_exists("buffer"));
        assert_eq!(true, project.buffer_exists("newbuffer"));

        Ok(())
    }

    #[test]
    fn test_buffer_rename_no_such_buffer() -> SimpleResult<()> {
        let mut project = H2Project::new("name", "1.0");

        // Rename a fake buffer
        assert!(project.buffer_rename("fakebuffer", "newbuffer").is_err());

        Ok(())
    }
}
