use redo::Command;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, SimpleError, bail};

use crate::datatype::H2Type;
use crate::project::h2project::H2Project;
use crate::project::actions::Action;

#[derive(Serialize, Deserialize, Debug)]
struct Forward {
    buffer: String,
    layer: String,
    datatype: H2Type,
    offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Backward {
    // TODO: Add undo for this
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Forward(Forward),
    Backward(Backward),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionEntryCreateFromType(State);

impl ActionEntryCreateFromType {
    pub fn new(buffer: &str, layer: &str, datatype: H2Type, offset: usize) -> Action {
        Action::EntryCreateFromType(
            ActionEntryCreateFromType(
                State::Forward(Forward {
                    buffer: buffer.to_string(),
                    layer: layer.to_string(),
                    datatype: datatype,
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

        // Do stuff with it
        project.entry_create_from_type(&forward.buffer, &forward.layer, forward.datatype.clone(), forward.offset)?;

        // Save the backward struct
        self.0 = State::Backward(Backward {});

        Ok(())
    }

    fn undo(&mut self, _project: &mut H2Project) -> SimpleResult<()> {
        bail!("Can't go backwards :(");
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use redo::Record;
//     use pretty_assertions::assert_eq;

//     use crate::transformation::{TransformHex, TransformBase64};
//     use crate::project::actions::{Action, ActionBufferCreateFromBytes};

//     #[test]
//     fn test_action() -> SimpleResult<()> {
//         let mut record: Record<Action> = Record::new(
//             H2Project::new("name", "1.0")
//         );

//         // Create a buffer with "JKLMN" encoded as hex then base64 - I alternate
//         // case to ensure it's undoing correctly:
//         // $ echo -ne '4a4B4c4D4e' | base64 NGE0QjRjNEQ0ZQ==
//         let action = ActionBufferCreateFromBytes::new("buffer", &b"NGE0QjRjNEQ0ZQ==".to_vec(), 0x80000000);
//         record.apply(action)?;
//         assert_eq!(b"NGE0QjRjNEQ0ZQ==".to_vec(), record.target().get_buffer("buffer")?.data);

//         // Undo the base64
//         let action = ActionBufferTransform::new("buffer", TransformBase64::standard());
//         record.apply(action)?;
//         assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().get_buffer("buffer")?.data);

//         let action = ActionBufferTransform::new("buffer", TransformHex::new());
//         record.apply(action)?;
//         assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

//         // Undo both
//         record.undo()?;
//         assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().get_buffer("buffer")?.data);

//         record.undo()?;
//         assert_eq!(b"NGE0QjRjNEQ0ZQ==".to_vec(), record.target().get_buffer("buffer")?.data);

//         // Redo them
//         record.redo()?;
//         assert_eq!(b"4a4B4c4D4e".to_vec(), record.target().get_buffer("buffer")?.data);

//         record.redo()?;
//         assert_eq!(b"JKLMN".to_vec(), record.target().get_buffer("buffer")?.data);

//         Ok(())
//     }

//     #[test]
//     fn test_action_fails_on_impossible_transform() -> SimpleResult<()> {
//         let mut record: Record<Action> = Record::new(
//             H2Project::new("name", "1.0")
//         );

//         // Definitely not hex
//         let action = ActionBufferCreateFromBytes::new("buffer", &b"abcxyz".to_vec(), 0x80000000);
//         record.apply(action)?;

//         // Try to unhex
//         let action = ActionBufferTransform::new("buffer", TransformHex::new());
//         assert!(record.apply(action).is_err());

//         // Make sure nothing changed
//         assert_eq!(b"abcxyz".to_vec(), record.target().get_buffer("buffer")?.data);

//         Ok(())
//     }

//     // #[test]
//     // fn test_action_fails_when_buffer_is_populated() -> SimpleResult<()> {
//     //     Ok(())
//     // }
// }
