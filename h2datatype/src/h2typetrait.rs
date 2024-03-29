use simple_error::{bail, SimpleResult};
use std::ops::Range;

use crate::{Alignment, Offset, ResolvedType, H2Type};
use generic_number::{Integer, Float, Character};

/// The core trait that makes a type into a type. All types must implement this.
///
/// # Type consumers
///
/// Consumers really don't need to know much about this trait - check out
/// [`H2Type`] instead. Everything in here can be consumed through that, and the
/// function documentation is targeted towards consumers, not implementors!
///
/// # Type developers
///
/// As a type developer, some of the traits must be implemented (obviously),
/// while others have sane defaults that you can rely on. In some cases, if the
/// default behaviour doesn't make sense for you or if you can implement it
/// faster, feel free to override it.
///
/// The `actual_size` function is particularly to implement for any types that
/// aren't 100% composed of other types. By default, we subtract the last
/// address of the last child from the first address of the first, but
/// simple types have no children.
pub trait H2TypeTrait {
    /// Can information (like size and children) be retrieved without context?
    ///
    /// I'm not entirely sure if this is meaningful anymore, but I'm keeping it
    /// anyways (for now)!
    fn is_static(&self) -> bool;

    /// The actual size, in bytes, of a type. This does not include alignment
    /// or padding.
    ///
    /// By default, this will resolve the type's children and subtract the
    /// start of the first child from the end of the last. For types with
    /// children that fully cover their range, this is a reasonable
    /// implementation, but there may be more efficient ways.
    ///
    /// Types without children - in general, [`crate::simple`]s - must also
    /// implement this. Without children, we can't tell.
    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        let children = self.children_with_range(offset)?;

        let first_range = match children.first() {
            Some((r, _, _)) => r,
            None => bail!("Can't calculate size with no child types"),
        };

        // This should never trigger, but just in case...
        let last_range = match children.last() {
            Some((r, _, _)) => r,
            None => bail!("Can't calculate size with no child types"),
        };

        Ok(last_range.end - first_range.start)
    }

    /// Get the aligned size.
    ///
    /// The default implementation is very likely fine for this.
    fn aligned_size(&self, offset: Offset, alignment: Alignment) -> SimpleResult<u64> {
        let range = self.range(offset, alignment)?;

        Ok(range.end - range.start)
    }

    /// Get the start and ending positions. To get the range without alignment,
    /// use [`Alignment::None`].
    ///
    /// The default implementation is very likely good. This is only
    /// implemented as a trait function because other trait functions (such as
    /// [`#resolve`]) use it.
    fn range(&self, offset: Offset, alignment: Alignment) -> SimpleResult<Range<u64>> {
        // Get the start and end
        let start = offset.position();
        let end   = start + self.actual_size(offset)?;

        // Do the rounding
        alignment.align(start..end)
    }

    /// Convert to a String.
    ///
    /// This String value is ultimately what is displayed by users, and should
    /// have any formatting that a user would want to see.
    fn to_display(&self, offset: Offset) -> SimpleResult<String>;

    /// Get "related" values - ie, what a pointer points to.
    fn related(&self, _offset: Offset) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }

    /// Get children of the type - that is, other types that make up this type.
    ///
    /// Some types have no children - we refer to those as
    /// [`crate::simple`]s.
    ///
    /// For types that DO have children, with one exception the types follow
    /// some guidelines:
    ///
    /// * Children are ordered and consecutive (with possible alignment).
    ///
    /// * Children take up the full type - that is, the type starts at the
    ///   first byte of the first child, and ends at the last byte of the last
    ///   child (with possible alignment).
    ///
    /// Provided your children follow those rules, [`#actual_size`] and
    /// [`#children_with_range`] and [`#resolve`] will work with their default
    /// implementations.
    fn children(&self, _offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        Ok(vec![])
    }

    /// Get a list of children with their associated (aligned) ranges.
    ///
    /// As notes in [`#children`], the default implementation assumes that the
    /// children are consecutive, adjacent, and make up the full parent type.
    /// As long as that's the case, the default implementation will work just
    /// fine.
    fn children_with_range(&self, offset: Offset) -> SimpleResult<Vec<(Range<u64>, Option<String>, H2Type)>> {
        let mut child_offset = offset;

        self.children(offset)?.into_iter().map(|(name, child)| {
            let range = child.aligned_range(child_offset)?;

            child_offset = offset.at(range.end);

            Ok((range, name, child.clone()))
        }).collect::<SimpleResult<Vec<_>>>()
    }

    /// Create a [`ResolvedType`] from this [`H2Type`] and context.
    ///
    /// A resolved type has all the values calculated, and is therefore very
    /// quick to use.
    fn resolve(&self, offset: Offset, alignment: Alignment, field_name: Option<String>) -> SimpleResult<ResolvedType> {
        Ok(ResolvedType {
            actual_range: self.range(offset, Alignment::None)?,
            aligned_range: self.range(offset, alignment)?,

            field_name: field_name,
            display: self.to_display(offset)?,

            // Resolve the children here and now
            children: self.children_with_range(offset)?.into_iter().map(|(range, name, child)| {
                // Errors here will be handled by the collect
                child.resolve(offset.at(range.start), name)
            }).collect::<SimpleResult<Vec<ResolvedType>>>()?,

            related: self.related(offset)?,

            as_string: self.to_string(offset).ok(),

            as_integer: self.to_integer(offset).ok(),
            as_float: self.to_float(offset).ok(),
            as_character: self.to_character(offset).ok(),
        })
    }

    /// Can this type output a [`String`] (in general)?
    ///
    /// Like [`#can_be_char`], this doesn't have to be perfect.
    fn can_be_string(&self) -> bool {
        false
    }

    /// Convert to a [`String`], if it's sensible for this type.
    fn to_string(&self, _offset: Offset) -> SimpleResult<String> {
        bail!("This type cannot be converted to a string");
    }

    fn can_be_integer(&self) -> bool {
        false
    }

    fn to_integer(&self, _offset: Offset) -> SimpleResult<Integer> {
        bail!("This type cannot be converted to an integer");
    }

    fn can_be_float(&self) -> bool {
        false
    }

    fn to_float(&self, _offset: Offset) -> SimpleResult<Float> {
        bail!("This type cannot be converted to a float");
    }

    fn can_be_character(&self) -> bool {
        false
    }

    fn to_character(&self, _offset: Offset) -> SimpleResult<Character> {
        bail!("This type cannot be converted to a character");
    }
}
