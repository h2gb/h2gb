use redo::Record;
use simple_error::SimpleResult;

use crate::actions::*;
use crate::datatype::{H2Type, ResolvedType};

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

pub fn create_entry(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: &H2Type, offset: usize, comment: Option<&str>) -> SimpleResult<ResolvedType> {
    // Resolve the entry
    let resolved = peek_entry(record, buffer, datatype, offset)?;

    // Commit it
    commit_entry(record, buffer, layer, resolved.clone(), Some(datatype.clone()), comment)?;

    Ok(resolved)
}
