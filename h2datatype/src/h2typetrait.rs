use simple_error::{bail, SimpleResult};
use std::ops::Range;

use generic_number::{Context, Integer, Float, Character};

use crate::{Alignment, Data, ResolvedType, H2Type};

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
/// The `base_size` function is particularly to implement for any types that
/// aren't 100% composed of other types. By default, we subtract the last
/// address of the last child from the first address of the first, but
/// simple types have no children.
pub trait H2TypeTrait {
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
    fn base_size(&self, context: Context) -> SimpleResult<usize> {
        let children = self.children_with_range(context)?;

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
    fn aligned_size(&self, context: Context, alignment: Alignment) -> SimpleResult<usize> {
        let range = self.range(context, alignment)?;

        Ok(range.end - range.start)
    }

    /// Get the start and ending positions. To get the range without alignment,
    /// use [`Alignment::None`].
    ///
    /// The default implementation is very likely good. This is only
    /// implemented as a trait function because other trait functions (such as
    /// [`#resolve`]) use it.
    fn range(&self, context: Context, alignment: Alignment) -> SimpleResult<Range<usize>> {
        // Get the start and end
        let start = context.position();
        let end   = start + self.base_size(context)?;

        // Do the rounding
        alignment.align(start..end)
    }

    /// Convert to a String.
    ///
    /// This String value is ultimately what is displayed by users, and should
    /// have any formatting that a user would want to see.
    fn to_display(&self, context: Context, data: &Data) -> SimpleResult<String>;

    /// Get "related" values - ie, what a pointer points to.
    fn related(&self, _context: Context) -> SimpleResult<Vec<(usize, H2Type)>> {
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
    /// Provided your children follow those rules, [`#base_size`] and
    /// [`#children_with_range`] and [`#resolve`] will work with their default
    /// implementations.
    fn children(&self, _context: Context) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        Ok(vec![])
    }

    /// Get a list of children with their associated (aligned) ranges.
    ///
    /// As notes in [`#children`], the default implementation assumes that the
    /// children are consecutive, adjacent, and make up the full parent type.
    /// As long as that's the case, the default implementation will work just
    /// fine.
    fn children_with_range(&self, context: Context) -> SimpleResult<Vec<(Range<usize>, Option<String>, H2Type)>> {
        let mut child_context = context;

        self.children(context)?.into_iter().map(|(name, child)| {
            let range = child.aligned_range(child_context)?;

            child_context = context.at(range.end);

            Ok((range, name, child.clone()))
        }).collect::<SimpleResult<Vec<_>>>()
    }

    /// Try to get name(s) for the field.
    ///
    /// The goal of this is to handle fields of types like constants or enums,
    /// where a numeric field can represent a string value.
    ///
    /// Since constants and enums are both optional and non-unique, this can
    /// return [`None`], or a [`Vec`] of possible values.
    ///
    /// When resolve() consumes this, it will assign a single name to
    /// `field_name`, or multiple names to `possible_field_names`.
    fn field_name_options(&self, _context: Context) -> SimpleResult<Option<Vec<String>>> {
        Ok(None)
    }

    /// Create a [`ResolvedType`] from this [`H2Type`] and context.
    ///
    /// A resolved type has all the values calculated, and is therefore very
    /// quick to use.
    fn resolve(&self, context: Context, alignment: Alignment, field_name_override: Option<String>, data: &Data) -> SimpleResult<ResolvedType> {
        Ok(ResolvedType {
            actual_range: self.range(context, Alignment::None)?,
            aligned_range: self.range(context, alignment)?,

            field_name: field_name_override,
            display: self.to_display(context, data)?,

            // Resolve the children here and now
            children: self.children_with_range(context)?.into_iter().map(|(range, name, child)| {
                // Errors here will be handled by the collect
                child.resolve(context.at(range.start), name, data)
            }).collect::<SimpleResult<Vec<ResolvedType>>>()?,

            related: self.related(context)?,

            as_string: self.to_string(context, data).ok(),

            as_integer: self.to_integer(context).ok(),
            as_float: self.to_float(context).ok(),
            as_character: self.to_character(context).ok(),
        })
    }

    /// Can this type output a [`String`] (in general)?
    ///
    /// Like [`#can_be_char`], this doesn't have to be perfect.
    fn can_be_string(&self) -> bool {
        false
    }

    /// Convert to a [`String`], if it's sensible for this type.
    fn to_string(&self, _context: Context, _data: &Data) -> SimpleResult<String> {
        bail!("This type cannot be converted to a string");
    }

    fn can_be_integer(&self) -> bool {
        false
    }

    fn to_integer(&self, _context: Context) -> SimpleResult<Integer> {
        bail!("This type cannot be converted to an integer");
    }

    fn can_be_float(&self) -> bool {
        false
    }

    fn to_float(&self, _context: Context) -> SimpleResult<Float> {
        bail!("This type cannot be converted to a float");
    }

    fn can_be_character(&self) -> bool {
        false
    }

    fn to_character(&self, _context: Context) -> SimpleResult<Character> {
        bail!("This type cannot be converted to a character");
    }
}
