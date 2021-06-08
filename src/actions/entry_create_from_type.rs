use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::datatype::H2Type;
use crate::project::h2project::H2Project;
use crate::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    buffer: String,
    layer: String,
    abstract_type: H2Type,
    offset: usize,
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
pub struct ActionEntryCreateFromType(State);

impl ActionEntryCreateFromType {
    pub fn new(buffer: &str, layer: &str, abstract_type: H2Type, offset: usize) -> Action {
        Action::EntryCreateFromType(
            ActionEntryCreateFromType(
                State::Forward(Forward {
                    buffer: buffer.to_string(),
                    layer: layer.to_string(),
                    abstract_type: abstract_type,
                    offset: offset,
                })
            )
        )
    }
}

impl Command for ActionEntryCreateFromType {
    type Target = H2Project;
    type Error = SimpleError;

    fn apply(&mut self, project: &mut H2Project) -> SimpleResult<()> {
        // Get the forward struct
        let forward = match &self.0 {
            State::Forward(f) => f,
            _                 => bail!("Failed to apply: action ended up in a broken undo/redo state"),
        };

        // Create the entry and saved the ResolvedType
        project.entry_create_from_type(&forward.buffer, &forward.layer, forward.abstract_type.clone(), forward.offset)?;

        // Save the backward struct
        // Gotta save enough to know where to find it
        self.0 = State::Backward(Backward {
            buffer: forward.buffer.clone(),
            layer: forward.layer.clone(),
            offset: forward.offset,
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

        let abstract_type = match entry.creator() {
            Some(a) => a,
            None => bail!("We undid the entry, but it did not contain the type we expected: {:?}", &entry),
        };

        // Save the backward struct
        self.0 = State::Forward(Forward {
            buffer: backward.buffer.clone(),
            layer: backward.layer.clone(),
            abstract_type: abstract_type,
            offset: backward.offset,
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

    use crate::datatype::{H2Number, LPString, ASCII, StrictASCII};
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
        let action = ActionEntryCreateFromType::new("buffer", "default", datatype, 0);
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
        let datatype = LPString::new(H2Number::new(GenericReader::U8, DefaultFormatter::new()), ASCII::new(StrictASCII::Strict))?;
        let action = ActionEntryCreateFromType::new("buffer", "default", datatype, 4);
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
        let action = ActionEntryCreateFromType::new("buffer", "default", datatype, 0);
        record.apply(action)?;

        // Make sure we can't overlap it
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        assert!(record.apply(ActionEntryCreateFromType::new("buffer", "default", datatype, 0)).is_err());

        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        assert!(record.apply(ActionEntryCreateFromType::new("buffer", "default", datatype, 1)).is_err());

        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        assert!(record.apply(ActionEntryCreateFromType::new("buffer", "default", datatype, 2)).is_err());

        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        assert!(record.apply(ActionEntryCreateFromType::new("buffer", "default", datatype, 3)).is_err());

        // Going off the end should also be an error
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        assert!(record.apply(ActionEntryCreateFromType::new("buffer", "default", datatype, 5)).is_err());

        // But 4, like the third bed, should be jussst right
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        assert!(record.apply(ActionEntryCreateFromType::new("buffer", "default", datatype, 4)).is_ok());

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
        let action = ActionEntryCreateFromType::new("buffer", "default", datatype, 0);
        record.apply(action)?;

        // Make sure we can't overlap it
        let datatype = H2Number::new(GenericReader::U32(Endian::Big), DefaultFormatter::new());
        assert!(record.apply(ActionEntryCreateFromType::new("buffer", "default", datatype, 0)).is_err());

        Ok(())
    }
}
