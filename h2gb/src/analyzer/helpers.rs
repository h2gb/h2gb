use redo::Record;
use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;
use h2datatype::{H2Type, ResolvedType};

use crate::actions::*;

pub fn peek_entry(record: &mut Record<Action>, buffer: &str, datatype: &H2Type, offset: usize) -> SimpleResult<ResolvedType> {
    record.target().buffer_get_or_err(buffer)?.peek(&datatype, offset)
}

pub fn commit_entry(record: &mut Record<Action>, buffer: &str, layer: &str, resolved_type: ResolvedType, origin: Option<H2Type>, comment: Option<&str>) -> SimpleResult<()> {
    // Save the offset
    let offset = resolved_type.actual_range.start;

    // Create the entry
    let create_action = ActionEntryCreate::new(buffer, layer, resolved_type, origin);
    record.apply(create_action)?;

    // Add a comment if one was given
    if let Some(c) = comment {
        let comment_action = ActionEntrySetComment::new(buffer, layer, offset as usize, Some(c.to_string()));
        record.apply(comment_action)?;
    }

    Ok(())
}

pub fn add_comment(record: &mut Record<Action>, buffer: &str, layer: &str, offset: usize, comment: &str) -> SimpleResult<()> {
    record.apply(ActionEntrySetComment::new(buffer, layer, offset as usize, Some(comment.to_string())))
}

pub fn create_entry(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: &H2Type, offset: usize, comment: Option<&str>) -> SimpleResult<ResolvedType> {
    // Resolve the entry
    let resolved = peek_entry(record, buffer, datatype, offset)?;

    // Commit it
    commit_entry(record, buffer, layer, resolved.clone(), Some(datatype.clone()), comment)?;

    Ok(resolved)
}

/// This is a helper function that creates a record, then returns it as a simple
/// u64 - I found myself doing this a lot.
pub fn create_entry_integer(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: &H2Type, offset: usize, comment: Option<&str>) -> SimpleResult<Integer> {
    if !datatype.can_be_integer() {
        bail!("Attempting to create a numeric entry from a non-numeric datatype");
    }

    create_entry(record, buffer, layer, datatype, offset, comment)?.as_integer.ok_or(
        SimpleError::new("Could not create entry as a u64 value")
    ).map_err( |e| SimpleError::new(format!("Could not interpret entry as an integer: {:?}", e)))
}

/// This is a helper function that creates a record, then returns it as a simple
/// String - I found myself doing this a lot.
pub fn create_entry_string(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: &H2Type, offset: usize, comment: Option<&str>) -> SimpleResult<String> {
    if !datatype.can_be_string() {
        bail!("Attempting to create a numeric entry from a non-numeric datatype");
    }

    create_entry(record, buffer, layer, datatype, offset, comment)?.as_string.ok_or(
        SimpleError::new("Could not create entry as a String value")
    )
}
