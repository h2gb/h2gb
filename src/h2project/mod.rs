use std::fmt;

use multi_vector::{MultiVector, AutoBumpyEntry};
use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::collections::HashMap;

pub mod h2buffer;

use h2buffer::{H2Buffer, H2BufferName, H2LayerInBuffer};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Entry {
    display: String,
    index: usize,
    size: usize,
}

impl AutoBumpyEntry for H2Entry {
    fn index(&self) -> usize { self.index }
    fn size(&self) -> usize { self.size }
}

// H2Project is the very core, and the root of undo. All actions will be taken
// via this object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Project {
    pub name: String,
    pub version: String,

    // Buffers that exist, indexed by their name; layers are stored in their
    // respective buffer
    buffers: HashMap<H2BufferName, H2Buffer>,

    // Entries that exist, index by buffer + layer; additionally, H2Entry
    // tracks its index and size within the buffer + layer
    entries: MultiVector<H2LayerInBuffer, H2Entry>,
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
}

impl fmt::Display for H2Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}, version: {}", self.name, self.version)
    }
}

// Buffer
impl H2Project {
    pub fn buffers(&self) -> &HashMap<H2BufferName, H2Buffer> {
        return &self.buffers;
    }

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
        if self.buffer_exists(name) {
            bail!("Buffer already exists");
        }

        // Go
        self.buffers.insert(name.to_string(), buffer);

        Ok(())
    }

    pub fn buffer_clone_shallow(&mut self, from: &str, to: &str) -> SimpleResult<()> {
        // Sanity check
        if self.buffer_exists(to) {
            bail!("Buffer already exists");
        }

        // Get the original
        let cloned_buffer = self.get_buffer(from)?.clone();

        // Insert it under the new name
        self.buffers.insert(to.to_string(), cloned_buffer);

        Ok(())
    }

    pub fn buffer_clone_deep(&mut self, _from: &str, _to: &str) -> SimpleResult<()> {
        // TODO: Implement this once we support layers/entries
        bail!("Not implemented");
    }

    pub fn buffer_can_be_removed(&self, name: &str) -> SimpleResult<()> {
        let buffer = self.get_buffer(name)?;

        if buffer.is_populated() {
            bail!("Buffer has data in it");
        }

        Ok(())
    }

    pub fn buffer_clone_partial(&mut self, from: &str, to: &str, start: usize, size: usize) -> SimpleResult<()> {
        // Get a handle to the buffer
        let from = self.get_buffer(from)?;

        // Make sure the new one doesn't exist yet
        if self.buffer_exists(to) {
            bail!("Target buffer already exists");
        }

        // Sanity check
        if start + size > from.data.len() {
            bail!("Editing data into buffer is too long");
        }

        if size == 0 {
            bail!("Can't copy zero bytes");
        }

        let new_buffer = H2Buffer::new(from.data[start..(start+size)].into(), from.base_address);
        self.buffer_insert(to, new_buffer)?;

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
}
