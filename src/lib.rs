#![feature(range_is_empty)]
#![allow(dead_code)] // TODO: Disable this

mod action;
mod h2project;
mod h2buffer;
mod h2layer;

// Actions we need:
// * load data as buffer
// * create layer
// * destroy layer
// * create entries
// * remove entries
// * transform
// * clone
// * undo
// * redo
// #[cfg(test)]
// mod tests {
//     use super::*;

//     use simple_error::SimpleResult;
//     // use multi_vector::MultiVector;

//     use h2project::H2Project;
//     use action::Action;
//     use redo::Record;
//     use pretty_assertions::assert_eq;

//     #[test]
//     fn test_project_rename() -> SimpleResult<()> {
//         let mut record: Record<action::Action> = Record::new(
//             H2Project::new("name", "1.0")
//         );

//         assert_eq!("name", record.target().name);

//         record.apply(Action::project_rename("newname"))?;
//         assert_eq!("newname", record.target().name);
//         record.undo()?;
//         assert_eq!("name", record.target().name);
//         record.redo()?;
//         assert_eq!("newname", record.target().name);

//         Ok(())
//     }

//     #[test]
//     fn test_project_buffer_create_empty() -> SimpleResult<()> {
//         let mut record: Record<action::Action> = Record::new(
//             H2Project::new("name", "1.0")
//         );

//         assert_eq!("name", record.target().name);

//         record.apply(Action::buffer_create_empty("name", 100, 0))?;
//         println!("Before:\n{}\n\n", serde_yaml::to_string(&record).unwrap());
//         record.undo()?;
//         // assert_eq!("name", record.target().name);
//         println!("After:\n{}\n\n", serde_yaml::to_string(&record).unwrap());

//         record.redo()?;
//         println!("After redo:\n{}\n\n", serde_yaml::to_string(&record).unwrap());
//         // record.redo()?;
//         // assert_eq!("newname", record.target().name);

//         Ok(())
//     }
// }
