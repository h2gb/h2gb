use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use h2datatype::{H2Type, ResolvedType};

use crate::actions::Action;
use crate::project::H2Project;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    buffer: String,
    layer: String,
    resolved_type: ResolvedType,
    origin: Option<H2Type>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Backward {
    buffer: String,
    layer: String,
    offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionEntryCreate(State);

impl ActionEntryCreate {
    pub fn new(buffer: &str, layer: &str, resolved_type: ResolvedType, origin: Option<H2Type>) -> Action {
        Action::EntryCreate(
            ActionEntryCreate(
                State::Forward(Forward {
                    buffer: buffer.to_string(),
                    layer: layer.to_string(),
                    resolved_type: resolved_type,
                    origin: origin,
                })
            )
        )
    }
}

impl Command for ActionEntryCreate {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Create the entry
        project
            .buffer_get_mut_or_err(&forward.buffer)?
            .layer_get_mut_or_err(&forward.layer)?
            .entry_create(forward.resolved_type.clone(), forward.origin.clone())?;

        // Save the backward struct
        self.0 = State::Backward(Backward {
            buffer: forward.buffer.clone(),
            layer: forward.layer.clone(),
            offset: forward.resolved_type.actual_range.start,
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let backward = match &self.0 {
            State::Backward(f) => f,
            _                  => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        project.buffer_get_mut(&backward.buffer).ok_or(
            SimpleError::new(format!("Could not find buffer: {}", backward.buffer))
        )?;

        // Remove the entry
        let (resolved_type, origin) = project
            .buffer_get_mut_or_err(&backward.buffer)?
            .layer_get_mut_or_err(&backward.layer)?
            .entry_remove(backward.offset)?
            .ok_or(SimpleError::new("Could not remove entry: not found"))?;

        // Save the backward struct
        self.0 = State::Forward(Forward {
            buffer: backward.buffer.clone(),
            layer: backward.layer.clone(),
            resolved_type: resolved_type,
            origin: origin,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use redo::Record;
    use pretty_assertions::assert_eq;

    use crate::actions::{Action, ActionBufferCreateFromBytes, ActionLayerCreate};

    use h2datatype::Data;
    use h2datatype::simple::numeric::H2Integer;
    use h2datatype::simple::string::LPString;

    use generic_number::{IntegerReader, CharacterReader, CharacterFormatter, Endian, DefaultFormatter};

    #[test]
    fn test_action_create_entry() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer + layer
        record.apply(ActionBufferCreateFromBytes::new("buffer", &b"\x01\x02\x03\x04\x0bHello World".to_vec(), 0))?;
        record.apply(ActionLayerCreate::new("buffer", "default"))?;

        // Create a numeric type
        let datatype = H2Integer::new(IntegerReader::U32(Endian::Big), DefaultFormatter::new_integer());
        let resolved = record.target()
            .buffer_get_or_err("buffer")?
            .peek(&datatype, 0, &Data::default())?;

        let action = ActionEntryCreate::new("buffer", "default", resolved, None);
        record.apply(action)?;

        // Make sure it's there
        let entry = record.target()
            .buffer_get_or_err("buffer")?
            .layer_get_or_err("default")?
            .entry_get(0)?.unwrap();
        assert_eq!(0x01020304, entry.resolved().as_integer.unwrap().as_usize().unwrap());
        assert_eq!(0..4, entry.resolved().aligned_range);

        // Retrieve it from the other side to make sure that works
        let entry = record.target()
            .buffer_get_or_err("buffer")?
            .layer_get_or_err("default")?
            .entry_get(3)?.unwrap();
        assert_eq!(0x01020304, entry.resolved().as_integer.unwrap().as_usize().unwrap());
        assert_eq!(0..4, entry.resolved().aligned_range);

        // Create a string type
        let datatype = LPString::new(
            IntegerReader::U8,
            CharacterReader::ASCII,
            CharacterFormatter::pretty_str_character(),
        )?;
        let resolved = record.target()
            .buffer_get_or_err("buffer")?
            .peek(&datatype, 4, &Data::default())?;

        let action = ActionEntryCreate::new("buffer", "default", resolved, None);
        record.apply(action)?;

        // Retrieve it
        let entry = record.target()
            .buffer_get_or_err("buffer")?
            .layer_get_or_err("default")?
            .entry_get(4)?.unwrap();
        assert_eq!("Hello World", entry.resolved().as_string.clone().unwrap());

        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(0)?
                .is_some());
        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(4)?
                .is_some());

        record.undo()?;

        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(0)?
                .is_some());
        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(4)?
                .is_none());

        record.undo()?;

        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(0)?
                .is_none());
        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(4)?
                .is_none());

        record.redo()?;

        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(0)?
                .is_some());
        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(4)?
                .is_none());

        record.redo()?;

        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(0)?
                .is_some());
        assert!(record.target()
                .buffer_get_or_err("buffer")?
                .layer_get_or_err("default")?
                .entry_get(4)?
                .is_some());

        Ok(())
    }

    #[test]
    fn test_overlap_on_same_layer() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer + layer
        record.apply(ActionBufferCreateFromBytes::new("buffer", &b"\x01\x02\x03\x04\x05\x06\x07\x08".to_vec(), 0))?;
        record.apply(ActionLayerCreate::new("buffer", "default"))?;

        // Create an entry
        let datatype = H2Integer::new(IntegerReader::U32(Endian::Big), DefaultFormatter::new_integer());

        // Resolve it
        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 0, &Data::default())?;

        // Insert it
        let action = ActionEntryCreate::new("buffer", "default", resolved, Some(datatype.clone()));
        record.apply(action)?;

        // Make sure we can't overlap it
        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 0, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default", resolved, Some(datatype.clone()))).is_err());

        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 1, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default", resolved, Some(datatype.clone()))).is_err());

        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 2, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default", resolved, Some(datatype.clone()))).is_err());

        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 3, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default", resolved, Some(datatype.clone()))).is_err());

        // Going off the end should also be an error
        // I need a bigger buffer to test this
        record.apply(ActionBufferCreateFromBytes::new("longbuffer", &b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(), 0))?;

        // Create the entry with the new, longer buffer, and insert it into the
        // original buffer
        let bad_resolved = record.target().buffer_get_or_err("longbuffer")?.peek(&datatype.clone(), 5, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default", bad_resolved, Some(datatype.clone()))).is_err());

        // But 4, like the third bed, should be jussst right
        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype.clone(), 4, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default", resolved, Some(datatype.clone()))).is_ok());

        Ok(())
    }

    #[test]
    fn test_overlap_on_different_layers() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer + a couple layers
        record.apply(ActionBufferCreateFromBytes::new("buffer", &b"\x01\x02\x03\x04\x05\x06\x07\x08".to_vec(), 0))?;
        record.apply(ActionLayerCreate::new("buffer", "default"))?;
        record.apply(ActionLayerCreate::new("buffer", "default2"))?;

        // Create an entry
        let datatype = H2Integer::new(IntegerReader::U32(Endian::Big), DefaultFormatter::new_integer());
        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 0, &Data::default())?;
        let action = ActionEntryCreate::new("buffer", "default", resolved, None);
        record.apply(action)?;

        // Make sure we can't overlap it on the same layer
        let datatype = H2Integer::new(IntegerReader::U32(Endian::Big), DefaultFormatter::new_integer());
        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 0, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default", resolved, None)).is_err());

        // But we can on the other
        let datatype = H2Integer::new(IntegerReader::U32(Endian::Big), DefaultFormatter::new_integer());
        let resolved = record.target().buffer_get_or_err("buffer")?.peek(&datatype, 0, &Data::default())?;
        assert!(record.apply(ActionEntryCreate::new("buffer", "default2", resolved, None)).is_ok());

        Ok(())
    }
}
