//! A data buffer, where the actual bytes are stored in an h2gb project.
//!
//! When data is created or imported by h2gb, the actual raw bytes end up here.
//! They can be transformed and edited (with restrictions).
//!
//! Conceptionally below the buffer is the context of a "layer", which is where
//! data is annotated and analyzed.
//!
//! All actions done on this buffer are designed to be reversible - anything
//! that changes is returned so it can be restored later, if desired. That's
//! a very important design principle!
//!
//! In general, you should never have mutable access to a buffer or project.
//! All changes should be made via actions, otherwise the undo/redo logic will
//! fall apart!
//!
//! In other words: DON'T USE THESE DIRECTLY, unless you're writing actions.

use std::collections::HashMap;
use std::ops::Range;
use std::{fmt, mem};

use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult, SimpleError};

use generic_number::Context;
use h2datatype::{H2Type, ResolvedType};
use h2transformation::Transformation;

use crate::project::H2Layer;

// H2Buffer holds the actual data, as well as its layers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H2Buffer {
    name: String,

    // TODO: Make these private, I think only tests are using them directly
    pub data: Vec<u8>,
    pub base_address: usize,

    // A list of transformations that this buffer has undergone
    transformations: Vec<Transformation>,

    layers: HashMap<String, H2Layer>,

    display_empty_addresses: bool,
    context_bytes: usize,
}

impl fmt::Display for H2Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Buffer: {} (base 0x{:x} / 0x{:x} bytes long)", self.name, self.base_address, self.data.len())?;
        writeln!(f, " Transformations:")?;
        for transformation in self.transformations.iter() {
            writeln!(f, " * {}", transformation)?;
        }
        writeln!(f)?;

        for (layer_name, layer) in &self.layers {
            writeln!(f, "Layer: {}", layer_name)?;

            // Start at offset 0, but process in entry-sized chunks
            let mut offset = 0;

            while offset < self.data.len() {
                match layer.entry_get(offset) {
                    Err(e) => {
                        write!(f, " 0x{:08x} - 0x{:08x}    Error getting entry: {:?}",
                                 offset,
                                 offset + 1,
                                 e,
                        )?;
                        offset += 1;
                    },
                    Ok(Some(entry)) => {
                        // Deal with the entry
                        let resolved = entry.resolved();
                        let actual_range = resolved.actual_range.start..resolved.actual_range.end;

                        let entry_byte_string: Vec<String> = self.data[actual_range.clone()].iter().take(self.context_bytes).map(|b| format!("{:02x}", b)).collect();
                        let entry_byte_string = entry_byte_string.join(" ");

                        write!(f, " 0x{:08x} - 0x{:08x}    {}   {}",
                                 actual_range.start + self.base_address,
                                 actual_range.end + self.base_address - 1,
                                 entry_byte_string,
                                 entry,
                        )?;

                        // Deal with comments on the entry
                        let comments = layer.comments_get(actual_range).unwrap();
                        if comments.len() == 0 {
                            writeln!(f, "")?;
                        } else {
                            let comments: Vec<String> = comments.iter().map(|c| c.to_string()).collect();
                            writeln!(f, " ; {}", comments.join(" / "))?;
                        }

                        // Deal with the padding / alignment
                        let alignment_range = (resolved.actual_range.end)..(resolved.aligned_range.end);
                        if !alignment_range.is_empty() {
                            let alignment_byte_string: Vec<String> = self.data[alignment_range.clone()].iter().map(|b| format!("{:02x}", b)).collect();
                            let alignment_byte_string = alignment_byte_string.join(" ");

                            write!(f, " 0x{:08x} - 0x{:08x}    {}    (padding / alignment)",
                                alignment_range.start + self.base_address,
                                alignment_range.end + self.base_address,
                                alignment_byte_string,
                            )?;

                            // Deal with comments on the alignment area
                            let comments = layer.comments_get(alignment_range).unwrap();
                            if comments.len() == 0 {
                                writeln!(f, "")?;
                            } else {
                                let comments: Vec<String> = comments.iter().map(|c| c.to_string()).collect();
                                writeln!(f, " ; {}", comments.join(" / "))?;
                            }
                        }

                        // Move to the next entry
                        offset = resolved.aligned_range.end;
                    },
                    Ok(None) => {
                        if self.display_empty_addresses {
                            // Handle the case where there are no entries here
                            write!(f, " 0x{:08x} - 0x{:08x}    {:02x}",
                                     offset + self.base_address,
                                     offset + self.base_address,
                                     self.data[offset],
                            )?;

                            match layer.comment_get(offset).unwrap() {
                                Some(comment) => writeln!(f, " ; {}", comment)?,
                                None =>          writeln!(f, "")?,
                            }
                        }

                        // Increment the offset
                        offset += 1;
                    }
                };
            }
        }

        Ok(())
    }
}

impl H2Buffer {
    /// Create a new buffer with the given data and base_address
    ///
    /// # Errors
    ///
    /// * Data must be at least
    pub fn new(name: &str, data: Vec<u8>, base_address: usize) -> SimpleResult<Self> {
        if data.len() == 0 {
            bail!("Can't create a buffer of zero length");
        }

        Ok(H2Buffer {
            name: name.to_string(),
            data: data,
            base_address: base_address,
            layers: HashMap::new(),
            transformations: Vec::new(),

            display_empty_addresses: true, // TODO: Figure out how to handle empty addresses
            context_bytes: 16, // TODO: Figure out how to configure this
        })
    }

    /// Get the length of the data buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Clone the buffer and data (but not the layers and entries).
    ///
    /// The base address can be preserved or changed as part of the copy. The
    /// actual data, as well as the list of transformations it's undergone, are
    /// preserved.
    ///
    /// # Errors
    ///
    /// * This will raise an error if the length of the original buffer is zero
    ///   (which obviously shouldn't be possible)
    // pub fn clone_shallow(&self, new_base_address: Option<usize>) -> SimpleResult<Self> {
    //     // Create the basics (use Self::new for consistent error checks)
    //     let mut cloned = Self::new(&self.name, self.data.clone(), new_base_address.unwrap_or(self.base_address))?;

    //     // Preserve the transformations
    //     cloned.transformations = self.transformations.clone();

    //     Ok(cloned)
    // }

    // // Not sure whether I want this...
    // pub fn clone_deep(&self) -> SimpleResult<()> {
    //     bail!("Not implemented");
    // }

    /// Clone a range of data within the buffer.
    ///
    /// This preserves the starting address of the data as the base address by
    /// default, or a new base address can be chosen.
    ///
    /// This does not clone layers or entries, and does not keep transformation
    /// history.
    ///
    /// # Errors
    ///
    /// * The `range` must not go off the end of the buffer
    /// * The `range` must not work out to zero bytes
    // pub fn clone_partial(&self, range: Range<usize>, new_base_address: Option<usize>) -> SimpleResult<Self> {
    //     // Sanity check
    //     if range.end > self.data.len() {
    //         bail!("Editing data into buffer is too long");
    //     }

    //     if range.is_empty() {
    //         bail!("Clone range cannot be empty");
    //     }

    //     let base_address = match new_base_address {
    //         Some(b) => b,
    //         None => self.base_address + range.start,
    //     };

    //     Self::new(&self.name, self.data[range].into(), base_address)
    // }

    pub fn byte_range(&self, range: Range<usize>) -> SimpleResult<&[u8]> {
        if range.end > self.data.len() {
            bail!("Invalid range");
        }

        Ok(&self.data[range])
    }

    /// Returns true if the buffer contains layers, entries, or any changes
    /// that could prevent it from being cleanly removed.
    ///
    /// # Errors
    ///
    /// * n/a
    pub fn is_populated(&self) -> bool {
        if self.layers.len() > 0 {
            return true;
        }

        return false;
    }

    /// Transform the buffer using an `Transformation`.
    ///
    /// # Errors
    ///
    /// * The buffer may not be populated
    /// * The transformation itself may fail (hex-decoding an odd-length string,
    ///   for eg)
    pub fn transform(&mut self, transformation: Transformation) -> SimpleResult<Vec<u8>> {
        // Sanity check
        if self.is_populated() {
            bail!("Buffer contains data");
        }

        // Transform the data - if this fails, nothing is left over
        let new_data = transformation.transform(&self.data)?;

        // Log the transformation
        self.transformations.push(transformation);

        // Replace it with the transformed, return the original
        Ok(mem::replace(&mut self.data, new_data))
    }

    /// Undo a transformation.
    ///
    /// Somewhat confusingly, this does NOT do an untransform - that's done by
    /// the `untranform` function.
    ///
    /// This is more of a "remove the transformation from the stack and replace
    /// the data". It's basically a hard undo, and assumes that it's being
    /// used correctly (hopefully through the action interface).
    ///
    /// # Errors
    ///
    /// In theory, this shouldn't be able to fail - failing during an undo is
    /// bad news!
    ///
    /// That being said, error conditions are:
    ///
    /// * If the buffer is populated, this will fail
    pub fn transform_undo(&mut self, original_data: Vec<u8>) -> SimpleResult<Transformation> {
        if self.is_populated() {
            bail!("Buffer contains data");
        }

        // Remove the transformation, or fail
        let transformation = match self.transformations.pop() {
            Some(t) => t,
            None => bail!("No transformations in the stack"),
        };

        // Replace the data after we've confirmed the transformation
        self.data = original_data;

        Ok(transformation)
    }

    /// Untransform the data, reversing the most recent transformation.
    pub fn untransform(&mut self) -> SimpleResult<(Vec<u8>, Transformation)> {
        if self.is_populated() {
            bail!("Buffer contains data");
        }

        // Make sure there's a transformation
        let transformation = match self.transformations.last() {
            Some(t) => t,
            None => bail!("Buffer has no transformations"),
        };

        // Attempt to untransform - fail before making any changes if it's not
        // possible
        let new_data = transformation.untransform(&self.data)?;

        // If we're here, it succeeded and we can remove the last element
        let transformation = match self.transformations.pop() {
            Some(t) => t,
            None => bail!("Transformation disappeared while untransforming!"),
        };

        // Replace it with the untransformed, return the original
        Ok((mem::replace(&mut self.data, new_data), transformation))
    }

    pub fn untransform_undo(&mut self, original_data: Vec<u8>, transformation: Transformation) -> SimpleResult<()> {
        if self.is_populated() {
            bail!("Buffer contains data");
        }

        // Replace the data; there's no need to save the forward data, we can
        // re-calculate that
        self.data = original_data;

        // Add the transformation back
        self.transformations.push(transformation);

        // We don't need to return anything here
        Ok(())
    }

    // pub fn edit(&mut self, data: Vec<u8>, offset: usize) -> SimpleResult<Vec<u8>> {
    //     // Get a handle to the buffer's data
    //     let buffer_data = &mut self.data;

    //     // Sanity check
    //     if offset + data.len() > buffer_data.len() {
    //         bail!("Editing data into buffer is too long");
    //     }

    //     if data.len() == 0 {
    //         bail!("Can't edit zero bytes");
    //     }

    //     // Splice in our data, get the original data back
    //     Ok(buffer_data.splice(offset..(offset+data.len()), data).collect())
    // }

    // pub fn rebase(&mut self, new_base_address: usize) -> SimpleResult<usize> {
    //     let old_base_address = self.base_address;
    //     self.base_address = new_base_address;

    //     Ok(old_base_address)
    // }

    // ** Everything below here is basically to operate on layers - create and
    //    remove, then a bunch of simple proxies to make it more ergonomic to
    //    deal with layers!

    pub fn layer_add(&mut self, layer: &str) -> SimpleResult<()> {
        // Get this up front, we won't be able to once we borrow self in the match
        let length = self.len();

        // Either insert, or error if there's already a layer there
        match self.layers.entry(layer.to_string()) {
            std::collections::hash_map::Entry::Occupied(_) => bail!("A layer named {} already exists in the buffer {}", layer, self.name),
            std::collections::hash_map::Entry::Vacant(v) => v.insert(H2Layer::new(layer, length)),
        };

        Ok(())
    }

    pub fn layer_remove(&mut self, layer: &str) -> SimpleResult<()> {
        let is_populated = match self.layers.get(layer) {
            Some(layer) => layer.is_populated(),
            None => bail!("Could not find layer {} in buffer {}", self.name, layer),
        };

        if is_populated {
            bail!("Cannot remove layer {} from buffer {} because it's not empty", layer, self.name);
        }

        match self.layers.remove(layer) {
            Some(_) => Ok(()),
            None => bail!("Failed to remove the layer"),
        }
    }

    pub fn layer_exists(&self, layer: &str) -> bool {
        self.layers.contains_key(layer)
    }

    pub fn layer_get(&self, layer: &str) -> Option<&H2Layer> {
        self.layers.get(layer)
    }

    pub fn layer_get_or_err(&self, layer: &str) -> SimpleResult<&H2Layer> {
        self.layer_get(layer).ok_or(
            SimpleError::new(format!("Could not find layer {}", layer))
        )
    }

    pub fn layer_get_mut(&mut self, layer: &str) -> Option<&mut H2Layer> {
        self.layers.get_mut(layer)
    }

    pub fn layer_get_mut_or_err(&mut self, layer: &str) -> SimpleResult<&mut H2Layer> {
        self.layer_get_mut(layer).ok_or(
            SimpleError::new(format!("Could not find layer {}", layer))
        )
    }

    pub fn peek(&self, abstract_type: &H2Type, offset: usize) -> SimpleResult<ResolvedType> {
        let offset = Context::new_at(&self.data, offset);

        abstract_type.resolve(offset, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use h2transformation::TransformHex;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new() -> SimpleResult<()> {
        let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x4000)?;
        assert_eq!(vec![0x41, 0x42, 0x43, 0x44], buffer.data);
        assert_eq!(0x4000, buffer.base_address);

        Ok(())
    }

    #[test]
    fn test_new_errors() -> SimpleResult<()> {
        assert!(H2Buffer::new("name", vec![], 0x4000).is_err());

        Ok(())
    }

    // #[test]
    // fn test_clone_shallow_same_base_address() -> SimpleResult<()> {
    //     let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x4000)?;
    //     let buffer = buffer.clone_shallow(None)?;
    //     assert_eq!(b"ABCD".to_vec(), buffer.data);
    //     assert_eq!(0x4000, buffer.base_address);

    //     Ok(())
    // }

    // #[test]
    // fn test_clone_shallow_new_base_address() -> SimpleResult<()> {
    //     let buffer = H2Buffer::new("name", b"ABCD".to_vec(), 0x4000)?;
    //     let buffer = buffer.clone_shallow(Some(0x8000))?;
    //     assert_eq!(b"ABCD".to_vec(), buffer.data);
    //     assert_eq!(0x8000, buffer.base_address);

    //     Ok(())
    // }

    // #[test]
    // fn test_clone_partial_same_base_address() -> SimpleResult<()> {
    //     let buffer = H2Buffer::new("name", b"ABCDEFGHIJKL".to_vec(), 0x4000)?;

    //     let buffer_start = buffer.clone_partial(0..4, None)?;
    //     assert_eq!(b"ABCD".to_vec(), buffer_start.data);
    //     assert_eq!(0x4000, buffer_start.base_address);

    //     let buffer_middle = buffer.clone_partial(2..10, None)?;
    //     assert_eq!(b"CDEFGHIJ".to_vec(), buffer_middle.data);
    //     assert_eq!(0x4002, buffer_middle.base_address);

    //     let buffer_end = buffer.clone_partial(8..12, None)?;
    //     assert_eq!(b"IJKL".to_vec(), buffer_end.data);
    //     assert_eq!(0x4008, buffer_end.base_address);

    //     Ok(())
    // }

    // #[test]
    // fn test_clone_partial_new_base_address() -> SimpleResult<()> {
    //     let buffer = H2Buffer::new("name", b"ABCDEFGHIJKL".to_vec(), 0x4000)?;

    //     let buffer_start = buffer.clone_partial(0..4, Some(0x8000))?;
    //     assert_eq!(b"ABCD".to_vec(), buffer_start.data);
    //     assert_eq!(0x8000, buffer_start.base_address);

    //     let buffer_middle = buffer.clone_partial(2..10, Some(0x8000))?;
    //     assert_eq!(b"CDEFGHIJ".to_vec(), buffer_middle.data);
    //     assert_eq!(0x8000, buffer_middle.base_address);

    //     let buffer_end = buffer.clone_partial(8..12, Some(0x8000))?;
    //     assert_eq!(b"IJKL".to_vec(), buffer_end.data);
    //     assert_eq!(0x8000, buffer_end.base_address);

    //     Ok(())
    // }

    // #[test]
    // fn test_clone_partial_bad_range() -> SimpleResult<()> {
    //     let buffer = H2Buffer::new("name", b"ABCDEFGHIJKL".to_vec(), 0x4000)?;

    //     assert!(buffer.clone_partial(0..0, None).is_err()); // Zero length
    //     assert!(buffer.clone_partial(4..0, None).is_err()); // Negative length
    //     assert!(buffer.clone_partial(0..100, None).is_err()); // Way off the end

    //     Ok(())
    // }

    #[test]
    fn test_is_populated() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_transform() -> SimpleResult<()> {
        let mut buffer = H2Buffer::new("name", b"41424344".to_vec(), 0x4000)?;
        let original = buffer.transform(TransformHex::new())?;

        assert_eq!(b"41424344".to_vec(), original);
        assert_eq!(b"ABCD".to_vec(), buffer.data);

        Ok(())
    }

    #[test]
    fn test_transform_bad_transformation() -> SimpleResult<()> {
        let mut buffer = H2Buffer::new("name", b"abc".to_vec(), 0x4000)?;
        assert!(buffer.transform(TransformHex::new()).is_err());

        Ok(())
    }

    #[test]
    fn test_transform_fails_when_populated() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_transform_undo() -> SimpleResult<()> {
        let mut buffer = H2Buffer::new("name", b"41424344".to_vec(), 0x4000)?;

        let original = buffer.transform(TransformHex::new())?;
        assert_eq!(b"ABCD".to_vec(), buffer.data);

        buffer.transform_undo(original)?;
        assert_eq!(b"41424344".to_vec(), buffer.data);

        assert!(buffer.transform_undo(b"TEST".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_untransform() -> SimpleResult<()> {
        let mut buffer = H2Buffer::new("name", b"4a4B4c4D".to_vec(), 0x4000)?;
        assert_eq!(b"4a4B4c4D".to_vec(), buffer.data);

        buffer.transform(TransformHex::new())?;
        assert_eq!(b"JKLM".to_vec(), buffer.data);

        // Note that the case normalizes
        let (data, transformation) = buffer.untransform()?;
        assert_eq!(b"4a4b4c4d".to_vec(), buffer.data);
        assert_eq!(b"JKLM".to_vec(), data);
        assert_eq!(transformation, TransformHex::new());

        Ok(())
    }

    #[test]
    fn test_untransform_undo() -> SimpleResult<()> {
        let mut buffer = H2Buffer::new("name", b"4a4B4c4D".to_vec(), 0x4000)?;
        assert_eq!(b"4a4B4c4D".to_vec(), buffer.data);

        buffer.transform(TransformHex::new())?;
        assert_eq!(b"JKLM".to_vec(), buffer.data);

        // Note that the case normalizes
        let (data, transformation) = buffer.untransform()?;
        assert_eq!(b"4a4b4c4d".to_vec(), buffer.data);
        assert_eq!(b"JKLM".to_vec(), data);
        assert_eq!(transformation, TransformHex::new());

        buffer.untransform_undo(data, transformation)?;
        assert_eq!(b"JKLM".to_vec(), buffer.data);

        Ok(())
    }

    // #[test]
    // fn test_edit() -> SimpleResult<()> {
    //     let mut buffer = H2Buffer::new("name", b"41424344".to_vec(), 0x4000)?;
    //     assert_eq!(b"41424344".to_vec(), buffer.data);

    //     let original = buffer.edit(b"ZZ".to_vec(), 0)?;
    //     assert_eq!(b"41".to_vec(), original);
    //     assert_eq!(b"ZZ424344".to_vec(), buffer.data);

    //     let original = buffer.edit(b"YY".to_vec(), 1)?;
    //     assert_eq!(b"Z4".to_vec(), original);
    //     assert_eq!(b"ZYY24344".to_vec(), buffer.data);

    //     let original = buffer.edit(b"~".to_vec(), 7)?;
    //     assert_eq!(b"4".to_vec(), original);
    //     assert_eq!(b"ZYY2434~".to_vec(), buffer.data);

    //     Ok(())
    // }

    // #[test]
    // fn test_edit_errors() -> SimpleResult<()> {
    //     let mut buffer = H2Buffer::new("name", b"41424344".to_vec(), 0x4000)?;
    //     assert_eq!(b"41424344".to_vec(), buffer.data);

    //     // Zero length
    //     assert!(buffer.edit(b"".to_vec(), 0).is_err());

    //     // Just overlaps the end
    //     assert!(buffer.edit(b"AAAA".to_vec(), 5).is_err());

    //     // Just overlaps the end
    //     assert!(buffer.edit(b"AAAAAAAAA".to_vec(), 0).is_err());

    //     // Starts off the end
    //     assert!(buffer.edit(b"A".to_vec(), 9).is_err());

    //     // Starts way off the end
    //     assert!(buffer.edit(b"A".to_vec(), 1000).is_err());

    //     Ok(())
    // }
}
