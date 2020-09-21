use std::mem;

use simple_error::{bail, SimpleResult};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use h2transformer::H2Transformation;
use std::ops::Range;

pub type H2BufferName = String;
// Create some types so we can tell what's what
pub type H2LayerName = String;
pub type H2LayerInBuffer = (H2BufferName, H2LayerName);

#[derive(Serialize, Deserialize, Debug)]
pub struct H2Layer {
    name: H2LayerName,
    buffer: H2BufferName,
}

// H2Buffer holds the actual data, as well as its layers
#[derive(Serialize, Deserialize, Debug)]
pub struct H2Buffer {
    pub data: Vec<u8>,
    pub base_address: usize,

    pub layers: HashMap<H2LayerName, H2Layer>,
    pub transformations: Vec<H2Transformation>,
}

impl H2Buffer {
    pub fn new(data: Vec<u8>, base_address: usize) -> SimpleResult<Self> {
        if data.len() == 0 {
            bail!("Can't create a buffer of zero length");
        }

        Ok(H2Buffer {
            data: data,
            base_address: base_address,
            layers: HashMap::new(),
            transformations: Vec::new(),
        })
    }

    pub fn clone_shallow(&self) -> SimpleResult<Self> {
        Self::new(self.data.clone(), self.base_address)
    }

    pub fn clone_deep(&self) -> SimpleResult<()> {
        // TODO: Implement this once we support layers/entries
        bail!("Not implemented");
    }

    pub fn clone_partial(&self, range: Range<usize>) -> SimpleResult<Self> {
        // Sanity check
        if range.end > self.data.len() {
            bail!("Editing data into buffer is too long");
        }

        Self::new(self.data[range].into(), self.base_address)
    }

    pub fn is_populated(&self) -> bool {
        if self.layers.len() > 0 {
            return true;
        }

        return false;
    }

    pub fn transform(&mut self, transformation: H2Transformation) -> SimpleResult<Vec<u8>> {
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

    pub fn transform_undo(&mut self, original_data: Vec<u8>) -> SimpleResult<H2Transformation> {
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

    pub fn untransform(&mut self) -> SimpleResult<(Vec<u8>, H2Transformation)> {
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

    pub fn untransform_undo(&mut self, original_data: Vec<u8>, transformation: H2Transformation) -> SimpleResult<()> {
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

    pub fn edit(&mut self, data: Vec<u8>, offset: usize) -> SimpleResult<Vec<u8>> {
        // Get a handle to the buffer's data
        let buffer_data = &mut self.data;

        // Sanity check
        if offset + data.len() > buffer_data.len() {
            bail!("Editing data into buffer is too long");
        }

        if data.len() == 0 {
            bail!("Can't edit zero bytes");
        }

        // Splice in our data, get the original data back
        Ok(buffer_data.splice(offset..(offset+data.len()), data).collect())
    }

    pub fn rebase(&mut self, new_base_address: usize) -> SimpleResult<usize> {
        let old_base_address = self.base_address;
        self.base_address = new_base_address;

        Ok(old_base_address)
    }
}
