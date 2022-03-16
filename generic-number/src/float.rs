use serde::{Serialize, Deserialize};
use std::{fmt, mem};
use std::cmp::Ordering;

/// Represents a floating point number - [`f32`] or [`f64`].
///
/// To use this, create a [`crate::FloatReader`] to read a [`crate::Context`].
/// That'll produce a `Float`. Then you can render it using a
/// renderer such as [`crate::ScientificFormatter`].
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a buffer
/// let buffer = b"\x40\x48\xf5\xc3".to_vec();
///
/// // Create a context that points to the start of the buffer
/// let context = Context::new_at(&buffer, 0);
///
/// // Create a reader that knows how to read a 32-bit float - this reader can
/// // be serialized and used later!
/// let reader = FloatReader::F32(Endian::Big);
///
/// // Read from the context into a generic number - this number can be
/// // serialized and used later!
/// let f = reader.read(context).unwrap();
///
/// // Display it using different formatters (these use the pretty defaults) -
/// // these formatters can also be serialized!
/// assert_eq!("3.14e0", ScientificFormatter::new_pretty().render_float(f));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Float {
    F32(f32),
    F64(f64),
}

impl From<f32>  for Float { fn from(o: f32) -> Self { Self::F32(o)  } }
impl From<f64>  for Float { fn from(o: f64) -> Self { Self::F64(o)  } }

impl Float {
    /// The size - in bytes - of the type.
    pub fn size(self) -> usize {
        match self {
            Self::F32(_)  => mem::size_of::<f32>(),
            Self::F64(_)  => mem::size_of::<f64>(),
        }
    }

    /// Internal function for implementing comparisons
    fn as_f64(self) -> f64 {
        match self {
            Self::F32(f) => f as f64,
            Self::F64(f) => f,
        }
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.as_f64().eq(&other.as_f64())
    }
}

// Floats don't get to be Eq
// impl Eq for Float { }

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_f64().partial_cmp(&other.as_f64())
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F32(v)  => fmt::Display::fmt(&v, f),
            Self::F64(v)  => fmt::Display::fmt(&v, f),
        }
    }
}

impl fmt::LowerExp for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F32(v)   => fmt::LowerExp::fmt(&v, f),
            Self::F64(v)  => fmt::LowerExp::fmt(&v, f),
        }
    }
}

impl fmt::UpperExp for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F32(v)   => fmt::UpperExp::fmt(&v, f),
            Self::F64(v)  => fmt::UpperExp::fmt(&v, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use simple_error::SimpleResult;
    use crate::Float;

    #[test]
    fn test_comparison() -> SimpleResult<()> {
        // Test ordering only, since we can't really do equality for floats
        assert!(Float::from(0f32) < Float::from(1f64));
        assert!(Float::from(0f64) < Float::from(1f32));
        assert!(Float::from(1f32) > Float::from(0f64));
        assert!(Float::from(1f64) > Float::from(0f32));

        Ok(())
    }
}
