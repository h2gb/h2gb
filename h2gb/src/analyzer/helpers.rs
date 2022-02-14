use redo::Record;
use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;
use h2datatype::{Data, H2Type, ResolvedType};

use crate::actions::*;

pub fn peek_entry(record: &mut Record<Action>, buffer: &str, datatype: &H2Type, offset: usize, data: &Data) -> SimpleResult<ResolvedType> {
    record.target().buffer_get_or_err(buffer)?.peek(&datatype, offset, data)
}

pub fn commit_entry(record: &mut Record<Action>, buffer: &str, layer: &str, resolved_type: ResolvedType, origin: Option<H2Type>, comment: Option<&str>) -> SimpleResult<()> {
    // Save the offset
    let offset = resolved_type.actual_range.start;

    // Create the entry
    let create_action = ActionEntryCreate::new(buffer, layer, resolved_type, origin);
    record.apply(create_action)?;

    // Add a comment if one was given
    if let Some(c) = comment {
        let comment_action = ActionEntrySetComment::new(buffer, layer, offset, Some(c.to_string()));
        record.apply(comment_action)?;
    }

    Ok(())
}

pub fn add_comment(record: &mut Record<Action>, buffer: &str, layer: &str, offset: usize, comment: &str) -> SimpleResult<()> {
    record.apply(ActionEntrySetComment::new(buffer, layer, offset, Some(comment.to_string())))
}

pub fn create_entry(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: &H2Type, offset: usize, comment: Option<&str>, data: &Data) -> SimpleResult<ResolvedType> {
    // Resolve the entry
    let resolved = peek_entry(record, buffer, datatype, offset, data)?;

    // Commit it
    commit_entry(record, buffer, layer, resolved.clone(), Some(datatype.clone()), comment)?;

    Ok(resolved)
}

/// This is a helper function that creates a record, then returns it as an
/// [`Integer`] - I found myself doing this a lot.
pub fn create_entry_integer(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: &H2Type, offset: usize, comment: Option<&str>, data: &Data) -> SimpleResult<Integer> {
    if !datatype.can_be_integer(data)? {
        bail!("Attempting to create a numeric entry from a non-numeric datatype: {:?}", datatype);
    }

    create_entry(record, buffer, layer, datatype, offset, comment, data)?.as_integer.ok_or(
        SimpleError::new("Could not create entry as an Integer value")
    ).map_err( |e| SimpleError::new(format!("Could not interpret entry as an integer: {:?}", e)))
}

/// This is a helper function that creates a record, then returns it as a simple
/// String - I found myself doing this a lot.
pub fn create_entry_string(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: &H2Type, offset: usize, comment: Option<&str>, data: &Data) -> SimpleResult<String> {
    if !datatype.can_be_string(data)? {
        bail!("Attempting to create a string entry from a non-string datatype: {:?}", datatype);
    }

    create_entry(record, buffer, layer, datatype, offset, comment, data)?.as_string.ok_or(
        SimpleError::new("Could not create entry as a String value")
    )
}
