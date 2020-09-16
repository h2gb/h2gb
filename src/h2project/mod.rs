// This implements the simplest "do a thing" stuff to H2Project. It does
// strict error checking - you can't remove a buffer with layers or entries, for
// example - but it also doesn't do any handholding, composite commands, etc.
//
// These are mostly mirrored by simple actions, whereas composite actions will
// do multiple of the simple actions.
use std::fmt;

use multi_vector::{MultiVector, AutoBumpyEntry};
use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::collections::HashMap;
use std::mem;

use h2transformer::H2Transformer;

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
    data: Vec<u8>,
    base_address: usize,

    layers: HashMap<H2LayerName, H2Layer>,
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

// Meta
impl H2Project {
    pub fn project_rename(&mut self, new_name: String) -> SimpleResult<String> {
        Ok(mem::replace(&mut self.name, new_name))
    }

    pub fn project_rename_undo(&mut self, old_name: String) -> SimpleResult<String> {
        Ok(mem::replace(&mut self.name, old_name))
    }
}

// // Buffer
// impl H2Project {
//     pub fn buffer_create(&mut self, data: BufferCreate) -> SimpleResult<BufferCreateUndo> {
//         if self.buffers.contains_key(&data.name) {
//             bail!("Buffer already exists");
//         }

//         self.buffers.insert(data.name.clone(), data.buffer);

//         Ok(BufferCreateUndo {
//             name: data.name,
//         })
//     }

//     pub fn buffer_create_undo(&mut self, data: BufferCreateUndo) {
//     }

//     // Only empty buffers can be removed
//     fn buffer_can_be_removed(&self, name: &H2BufferName) -> SimpleResult<()> {
//         // Check if the buffer exists
//         if let Some(b) = self.buffers.get(name) {
//             // Don't allow populated buffers to be removed
//             if b.layers.len() > 0 {
//                 bail!("Buffer still has data in it");
//             }

//             return Ok(())
//         } else {
//             bail!("No such buffer");
//         }
//     }

//     pub fn buffer_delete(&mut self, name: &H2BufferName) -> SimpleResult<H2Buffer> {
//         match self.buffer_can_be_removed(name) {
//             Ok(()) => {
//                 match self.buffers.remove(name) {
//                     Some(b) => Ok(b),
//                     None => Err("No such buffer".into()),
//                 }
//             }
//             Err(e) => Err(e),
//         }
//     }

//     pub fn buffer_delete_undo(&mut self, buffer: H2Buffer) -> SimpleResult<()> {
//         Ok(())
//     }

//     // Only buffers that haven't been populated yet can be transformed.
//     fn buffer_can_be_transformed(buffer: &H2Buffer, transformation: &H2Transformer) -> SimpleResult<()> {
//         // Don't allow populated buffers to be removed
//         if buffer.layers.len() > 0 {
//             bail!("Buffer has data");
//         }

//         if !transformation.can_transform(&buffer.data) {
//             bail!("Transformation won't work on this buffer");
//         }

//         return Ok(())
//     }

//     pub fn buffer_transform(&mut self, name: &H2BufferName, transformation: &H2Transformer) -> SimpleResult<Vec<u8>> {
//         match self.buffers.get_mut(name) {
//             Some(buffer) => {
//                 // Ensure it can be transformed
//                 Self::buffer_can_be_transformed(&buffer, transformation)?;

//                 // Transform to get the new data
//                 let new_data = transformation.transform(&buffer.data)?;

//                 // Swap out the old data with the new
//                 let old_data = mem::replace(&mut buffer.data, new_data);

//                 // Return the old data (so we can undo it later)
//                 Ok(old_data)
//             },
//             None => {
//                 bail!("No such buffer")
//             }
//         }
//     }

//     pub fn buffer_transform_undo(&mut self, name: &H2BufferName, original_data: Vec<u8>) -> SimpleResult<()> {
//         match self.buffers.get_mut(name) {
//             Some(buffer) => {
//                 buffer.data = original_data;

//                 Ok(())
//             },
//             None => {
//                 bail!("No such buffer")
//             }
//         }
//     }
// }

// // Layer
// impl H2Project {
//     pub fn layer_create(layer: H2Layer) {
//     }

//     pub fn layer_remove(layer: H2LayerInBuffer) {
//     }
// }

// // Entry
// impl H2Project {
//     pub fn entries_create(entries: Vec<H2Entry>) {
//     }

//     pub fn entries_remove() {
//     }

//     pub fn entry_update() {
//     }
// }
#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_project_rename() -> SimpleResult<()> {
        Ok(())
    }
}
