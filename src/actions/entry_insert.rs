use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::bumpy_vector::AutoBumpyEntry; // So we can use 'range()'
use crate::project::{H2Project, H2Entry};
use crate::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    buffer: String,
    layer: String,
    entry: H2Entry,
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
pub struct ActionEntryInsert(State);

impl ActionEntryInsert {
    pub fn new(buffer: &str, layer: &str, entry: H2Entry) -> Action {
        Action::EntryInsert(
            ActionEntryInsert(
                State::Forward(Forward {
                    buffer: buffer.to_string(),
                    layer: layer.to_string(),
                    entry: entry,
                })
            )
        )
    }
}

impl Command for ActionEntryInsert {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Create the entry and saved the ResolvedType
        project.entry_insert(&forward.buffer, &forward.layer, forward.entry.clone())?;

        // Save the backward struct
        // Gotta save enough to know where to find it
        self.0 = State::Backward(Backward {
            buffer: forward.buffer.clone(),
            layer: forward.layer.clone(),
            offset: forward.entry.range().start,
        });

        Ok(())
    }

    fn undo(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let backward = match &self.0 {
            State::Backward(f) => f,
            _                  => bail!("Failed to undo: action ended up in a broken undo/redo state"),
        };

        // Do stuff with it
        // Remove the entry
        let entry = match project.entry_remove(&backward.buffer, &backward.layer, backward.offset) {
            Some(e) => e,
            None => bail!("Could not remove entry: not found"),
        };

        // Save the backward struct
        self.0 = State::Forward(Forward {
            buffer: backward.buffer.clone(),
            layer: backward.layer.clone(),
            entry: entry,
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

    use crate::datatype::simple::H2Number;
    use crate::datatype::composite::string::LPString;
    use crate::generic_number::{GenericReader, Endian, DefaultFormatter};

    #[test]
    fn test_action_create_entry() -> SimpleResult<()> {
        let mut record: Record<Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        // Create a buffer + layer
        record.apply(ActionBufferCreateFromBytes::new("buffer", &b"\x01\x02\x03\x04\x0bHello World".to_vec(), 0))?;
        record.apply(ActionLayerCreate::new("buffer", "default"))?;

        // Create a numeric type
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        let entry = record.target().entry_create("buffer", datatype, 0)?;
        let action = ActionEntryInsert::new("buffer", "default", entry);
        record.apply(action)?;

        // Make sure it's there
        let entry = record.target().entry_get("buffer", "default", 0).unwrap();
        assert_eq!(0x01020304, entry.resolved().as_number.unwrap().as_u64().unwrap());
        assert_eq!(0..4, entry.resolved().aligned_range);

        // Retrieve it from the other side to make sure that works
        let entry = record.target().entry_get("buffer", "default", 3).unwrap();
        assert_eq!(0x01020304, entry.resolved().as_number.unwrap().as_u64().unwrap());
        assert_eq!(0..4, entry.resolved().aligned_range);

        // Create a string type
        let datatype = LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new_ascii(),
        )?;
        let entry = record.target().entry_create("buffer", datatype, 4)?;
        let action = ActionEntryInsert::new("buffer", "default", entry);
        record.apply(action)?;

        // Retrieve it
        let entry = record.target().entry_get("buffer", "default", 4).unwrap();
        assert_eq!("Hello World", entry.resolved().as_string.clone().unwrap());

        record.undo()?;

        assert!(record.target().entry_get("buffer", "default", 0).is_some());
        assert!(record.target().entry_get("buffer", "default", 4).is_none());

        record.undo()?;

        assert!(record.target().entry_get("buffer", "default", 0).is_none());
        assert!(record.target().entry_get("buffer", "default", 4).is_none());

        record.redo()?;

        assert!(record.target().entry_get("buffer", "default", 0).is_some());
        assert!(record.target().entry_get("buffer", "default", 4).is_none());

        record.redo()?;

        assert!(record.target().entry_get("buffer", "default", 0).is_some());
        assert!(record.target().entry_get("buffer", "default", 4).is_some());

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
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());

        // Create an entry that starts at the beginning
        let entry = record.target().entry_create("buffer", datatype.clone(), 0)?;
        let action = ActionEntryInsert::new("buffer", "default", entry);
        record.apply(action)?;

        // Make sure we can't overlap it
        let entry = record.target().entry_create("buffer", datatype.clone(), 0)?;
        assert!(record.apply(ActionEntryInsert::new("buffer", "default", entry)).is_err());

        let entry = record.target().entry_create("buffer", datatype.clone(), 1)?;
        assert!(record.apply(ActionEntryInsert::new("buffer", "default", entry)).is_err());

        let entry = record.target().entry_create("buffer", datatype.clone(), 2)?;
        assert!(record.apply(ActionEntryInsert::new("buffer", "default", entry)).is_err());

        let entry = record.target().entry_create("buffer", datatype.clone(), 3)?;
        assert!(record.apply(ActionEntryInsert::new("buffer", "default", entry)).is_err());

        // Going off the end should also be an error
        // I need a bigger buffer to test this
        record.apply(ActionBufferCreateFromBytes::new("longbuffer", &b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(), 0))?;
        // Create the entry with the new, longer buffer
        let entry = record.target().entry_create("longbuffer", datatype.clone(), 5)?;
        // And insert it into the old buffer
        assert!(record.apply(ActionEntryInsert::new("buffer", "default", entry)).is_err());

        // But 4, like the third bed, should be jussst right
        let entry = record.target().entry_create("buffer", datatype.clone(), 4)?;
        assert!(record.apply(ActionEntryInsert::new("buffer", "default", entry)).is_ok());

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
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        let entry = record.target().entry_create("buffer", datatype, 0)?;
        let action = ActionEntryInsert::new("buffer", "default", entry);
        record.apply(action)?;

        // Make sure we can't overlap it on the same layer
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        let entry = record.target().entry_create("buffer", datatype, 0)?;
        assert!(record.apply(ActionEntryInsert::new("buffer", "default", entry)).is_err());

        // But we can on the other
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        let entry = record.target().entry_create("buffer", datatype, 0)?;
        assert!(record.apply(ActionEntryInsert::new("buffer", "default2", entry)).is_ok());

        Ok(())
    }
}
