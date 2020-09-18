use std::{fmt, mem};

use multi_vector::{MultiVector, AutoBumpyEntry};
use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::collections::HashMap;
use h2transformer::H2Transformation;

// Create some types so we can tell what's what
type H2BufferName = String;
type H2LayerName = String;
type H2LayerInBuffer = (H2BufferName, H2LayerName);

// H2Layer is conceptually a list of entries with a name associated. We don't
// actually put the data in here, because we need interaction between multiple
// layers to happen, so we store the actual entries in H2Project indexed by
// the same name
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Layer {
    name: H2LayerName,
    buffer: H2BufferName,
}

// H2Buffer holds the actual data, as well as its layers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Buffer {
    pub data: Vec<u8>,
    pub base_address: usize,

    layers: HashMap<H2LayerName, H2Layer>,
    transformations: Vec<H2Transformation>,
}

impl H2Buffer {
    pub fn new(data: Vec<u8>, base_address: usize) -> Self {
        H2Buffer {
            data: data,
            base_address: base_address,
            layers: HashMap::new(),
            transformations: Vec::new(),
        }
    }

    pub fn is_populated(&self) -> bool {
        if self.layers.len() > 0 {
            return true;
        }

        return false;
    }
}


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

// pub struct BufferCreate {
//     pub name: H2BufferName,
//     pub buffer: H2Buffer,
// }

// pub struct BufferCreateUndo {
//     pub name: H2BufferName,
// }

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

    fn get_buffer_mut(&mut self, name: &str) -> SimpleResult<&mut H2Buffer> {
        match self.buffers.get_mut(name) {
            Some(b) => Ok(b),
            None => bail!("Buffer {} not found", name),
        }
    }

    pub fn buffer_insert(&mut self, name: &str, buffer: H2Buffer) -> SimpleResult<()> {
        if self.buffers.contains_key(name) {
            bail!("Buffer already exists");
        }

        self.buffers.insert(name.to_string(), buffer);

        Ok(())
    }

    pub fn buffer_can_be_removed(&self, name: &str) -> SimpleResult<()> {
        let buffer = self.get_buffer(name)?;

        if buffer.is_populated() {
            bail!("Buffer has data in it");
        }

        Ok(())
    }

    pub fn buffer_remove(&mut self, name: &str) -> SimpleResult<H2Buffer> {
        self.buffer_can_be_removed(name)?;

        match self.buffers.remove(name) {
            Some(b) => Ok(b),
            None => bail!("Buffer not found"),
        }
    }

    pub fn buffer_exists(&mut self, name: &str) -> bool {
        self.buffers.contains_key(name)
    }

    pub fn buffer_transform(&mut self, name: &str, transformation: H2Transformation) -> SimpleResult<Vec<u8>> {
        let buffer = self.get_buffer_mut(name)?;
        if buffer.is_populated() {
            bail!("Buffer {} contains data", name);
        }

        // Transform the data
        let new_data = transformation.transform(&buffer.data)?;

        // Log the transformation
        buffer.transformations.push(transformation);

        // Replace it with the transformed, return the original
        Ok(mem::replace(&mut buffer.data, new_data))
    }

    pub fn buffer_transform_undo(&mut self, name: &str, original_data: Vec<u8>) -> SimpleResult<H2Transformation> {
        let buffer = self.get_buffer_mut(name)?;
        if buffer.is_populated() {
            bail!("Buffer {} contains data", name);
        }

        // Remove the transformation
        let transformation = match buffer.transformations.pop() {
            Some(t) => t,
            None => bail!("No transformations in the stack"),
        };

        // Replace the data
        buffer.data = original_data;

        Ok(transformation)
    }

    pub fn buffer_untransform(&mut self, name: &str) -> SimpleResult<(Vec<u8>, H2Transformation)> {
        let buffer = self.get_buffer_mut(name)?;
        if buffer.is_populated() {
            bail!("Buffer {} contains data", name);
        }

        // Make sure there's a transformation
        let transformation = match buffer.transformations.last() {
            Some(t) => t,
            None => bail!("Buffer {} has no transformations", name),
        };

        // Attempt to untransform
        let new_data = transformation.untransform(&buffer.data)?;

        // If we're here, it succeeded and we can remove the last element
        let transformation = match buffer.transformations.pop() {
            Some(t) => t,
            None => bail!("Transformation disappeared while untransforming!"),
        };

        // Replace it with the untransformed, return the original
        Ok((mem::replace(&mut buffer.data, new_data), transformation))
    }

    pub fn buffer_untransform_undo(&mut self, name: &str, original_data: Vec<u8>, transformation: H2Transformation) -> SimpleResult<Vec<u8>> {
        let buffer = self.get_buffer_mut(name)?;
        if buffer.is_populated() {
            bail!("Buffer {} contains data", name);
        }

        // Replace the data
        let untransformed_data = mem::replace(&mut buffer.data, original_data);

        // Add the transformation back
        buffer.transformations.push(transformation);

        // Replace it with the untransformed, return the original
        Ok(untransformed_data)
    }
}
