use std::ops::Range;

use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use generic_number::{Context, Integer, Float, Character};

use crate::{H2TypeTrait, Alignment, Data, ResolvedType};
use crate::simple::*;
use crate::simple::network::*;
use crate::simple::numeric::*;
use crate::simple::string::*;
use crate::composite::*;

/// An enum used to multiplex between the various types.
///
/// Consumers of this library probably won't have to use this directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2InnerType {
    // Simple
    //H2Pointer(H2Pointer),
    Rgb(Rgb),
    H2Bitmask(H2Bitmask),
    H2Enum(H2Enum),
    H2UUID(H2UUID),
    H2Blob(H2Blob),

    // Numeric
    H2Character(H2Character),
    H2Float(H2Float),
    H2Integer(H2Integer),

    // Network
    IPv4(IPv4),
    IPv6(IPv6),
    MacAddress(MacAddress),
    MacAddress8(MacAddress8),

    // Strings
    H2String(H2String),
    NTString(NTString),
    LPString(LPString),

    // Composite
    H2Array(H2Array),
    H2Struct(H2Struct),

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2TypeType {
    Inline(H2InnerType),
    Named(String),
}

/// The core of this crate - defines any type of value abstractly.
///
/// In general, when consuming this crate, you probably won't be creating an
/// `H2Type` directly; rather, use the `new()` or `new_aligned()` function of
/// any of the various types defined in [`crate::simple`],
/// [`crate::composite`], or [`crate::composite::string`].
/// Those `new()` functions return an `H2Type`.
///
/// Please note that many of the functions here are very expensive, because
/// they have to read the object and iterate every time they're called. If you
/// call `resolve()`, a static version will be created with the fields pre-
/// calculated.
///
/// In terms of implementation, this basically passes everything through to
/// [`H2TypeTrait`]. The biggest reason for having this layer above the trait
/// is to store an alignment value.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Type {
    pub field: H2TypeType,
    pub alignment: Alignment,
}

impl H2Type {
    pub fn new_inline(alignment: Alignment, field: H2InnerType) -> Self {
        Self {
            field: H2TypeType::Inline(field),
            alignment: alignment,
        }
    }

    pub fn new_named_aligned(alignment: Alignment, name: &str, data: &Data) -> SimpleResult<Self> {
        if !data.types.contains(name) {
            bail!("No such named type: {}", name);
        }

        Ok(Self {
            field: H2TypeType::Named(name.to_string()),
            alignment: alignment,
        })
    }

    pub fn new_named(name: &str, data: &Data) -> SimpleResult<Self> {
        Self::new_named_aligned(Alignment::None, name, data)
    }

    fn field<'a>(&'a self, data: &'a Data) -> SimpleResult<&'a H2InnerType> {
        // XXX: Handle infinite recursion
        match &self.field {
            H2TypeType::Inline(t) => Ok(t),
            H2TypeType::Named(n) => data.types.get(&n)?.get().field(data),
        }
    }

    fn field_type<'a>(&'a self, data: &'a Data) -> SimpleResult<&'a dyn H2TypeTrait> {
        Ok(match self.field(data)? {
            // Simple
            //H2InnerType::H2Pointer(t) => t,
            H2InnerType::Rgb(t)       => t,
            H2InnerType::H2Bitmask(t) => t,
            H2InnerType::H2Enum(t)    => t,
            H2InnerType::H2UUID(t)    => t,
            H2InnerType::H2Blob(t)    => t,

            // Numeric
            H2InnerType::H2Float(t)     => t,
            H2InnerType::H2Character(t) => t,
            H2InnerType::H2Integer(t)   => t,

            // Network
            H2InnerType::IPv4(t)        => t,
            H2InnerType::IPv6(t)        => t,
            H2InnerType::MacAddress(t)  => t,
            H2InnerType::MacAddress8(t) => t,

            // Complex
            H2InnerType::H2Array(t)   => t,
            H2InnerType::H2Struct(t)  => t,

            // Strings
            H2InnerType::H2String(t)  => t,
            H2InnerType::NTString(t)  => t,
            H2InnerType::LPString(t)  => t,
        })
    }

    /// Get the size of just the field - no alignment included.
    ///
    /// Note that if the type has children (such as a
    /// [`crate::composite::H2Array`], the alignment on THAT is
    /// included since that's part of the actual object.
    pub fn base_size(&self, context: Context, data: &Data) -> SimpleResult<usize> {
        self.field_type(data)?.base_size(context, data)
    }

    /// Get the size of the field, including the alignment.
    pub fn aligned_size(&self, context: Context, data: &Data) -> SimpleResult<usize> {
        self.field_type(data)?.aligned_size(context, self.alignment, data)
    }

    /// Get the [`Range<usize>`] that the type will cover, starting at the
    /// given [`Context`], if it can be known, without adding padding.
    pub fn actual_range(&self, context: Context, data: &Data) -> SimpleResult<Range<usize>> {
        self.field_type(data)?.range(context, Alignment::None, data)
    }

    /// Get the [`Range<usize>`] that the type will cover, with padding.
    pub fn aligned_range(&self, context: Context, data: &Data) -> SimpleResult<Range<usize>> {
        self.field_type(data)?.range(context, self.alignment, data)
    }

    /// Get *related* nodes - ie, other fields that a pointer points to
    pub fn related(&self, context: Context, data: &Data) -> SimpleResult<Vec<(usize, H2Type)>> {
        self.field_type(data)?.related(context)
    }

    /// Get the types that make up the given type.
    ///
    /// Some types don't have children, they are essentially leaf notes. Others
    /// (such as [`H2Array`] and
    /// [`NTString`]) do.
    pub fn children(&self, context: Context, data: &Data) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        self.field_type(data)?.children(context)
    }

    /// Resolve this type into a concrete type.
    ///
    /// Once a type is resolved, the size, range, data, string value, and so on
    /// are "written in stone", so to speak, which means they no longer need to
    /// be calculated.
    pub fn resolve(&self, context: Context, name: Option<String>, data: &Data) -> SimpleResult<ResolvedType> {
        self.field_type(data)?.resolve(context, self.alignment, name, data)
    }

    /// Get a user-consumeable string
    pub fn to_display(&self, context: Context, data: &Data) -> SimpleResult<String> {
        self.field_type(data)?.to_display(context, data)
    }

    /// Can this value represent a [`String`]?
    pub fn can_be_string(&self, data: &Data) -> SimpleResult<bool> {
        Ok(self.field_type(data)?.can_be_string())
    }

    /// Try to convert to a [`String`].
    pub fn to_string(&self, context: Context, data: &Data) -> SimpleResult<String> {
        self.field_type(data)?.to_string(context, data)
    }

    pub fn can_be_integer(&self, data: &Data) -> SimpleResult<bool> {
        Ok(self.field_type(data)?.can_be_integer())
    }

    pub fn to_integer(&self, context: Context, data: &Data) -> SimpleResult<Integer> {
        self.field_type(data)?.to_integer(context)
    }

    pub fn can_be_float(&self, data: &Data) -> SimpleResult<bool> {
        Ok(self.field_type(data)?.can_be_float())
    }

    pub fn to_float(&self, context: Context, data: &Data) -> SimpleResult<Float> {
        self.field_type(data)?.to_float(context)
    }

    pub fn can_be_character(&self, data: &Data) -> SimpleResult<bool> {
        Ok(self.field_type(data)?.can_be_character())
    }

    pub fn to_character(&self, context: Context, data: &Data) -> SimpleResult<Character> {
        self.field_type(data)?.to_character(context)
    }
}
