use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};
use std::mem;

use crate::{Context, Endian, Float};

/// Defines how data is read from a [`Context`] to produce a [`GenericNumber`].
///
/// Importantly, this can be serialized, which means it can be stored and
/// re-used in the future.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FloatReader {
    /// 32-bit float
    F32(Endian),

    /// 64-bit float
    F64(Endian),
}

impl FloatReader {
    /// Read the chosen value at the given [`Context`].
    ///
    /// The `Context` has the offset embedded, and the [`FloatReader`] has the
    /// [`Endian`] built-in.
    ///
    /// If successful, this creates a [`Float`], whose datatype will
    /// match the type that we chose in this struct.
    pub fn read(self, context: Context) -> SimpleResult<Float> {
        match self {
            Self::F32(endian) => Ok(Float::from(context.read_f32(endian)?)),
            Self::F64(endian) => Ok(Float::from(context.read_f64(endian)?)),
        }
    }

    /// The size - in bytes - that will be read by [`Self::read`].
    pub fn size(self) -> usize {
        match self {
            Self::F32(_)  => mem::size_of::<f32>(),
            Self::F64(_)  => mem::size_of::<f64>(),
        }
    }
}

#[cfg(test)]
mod tests {
}
