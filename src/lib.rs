#![allow(dead_code)] // TODO: Disable this

mod h2project;
mod action;

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
#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    // use multi_vector::MultiVector;

    use h2project::H2Project;
    use action::Action;
    use action::null::NullAction;
    use action::project_rename::ActionProjectRename;
    use action::buffer_create_empty::ActionBufferCreateEmpty;
    use redo::Record;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_project_rename() -> SimpleResult<()> {
        let mut record: Record<action::Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert_eq!("name", record.target().name);

        record.apply(Action::ProjectRename("newname".into()))?;
        assert_eq!("newname", record.target().name);
        record.undo()?;
        assert_eq!("name", record.target().name);
        record.redo()?;
        assert_eq!("newname", record.target().name);

        Ok(())
    }

    #[test]
    fn test_project_buffer_create_empty() -> SimpleResult<()> {
        let mut record: Record<action::Action> = Record::new(
            H2Project::new("name", "1.0")
        );

        assert_eq!("name", record.target().name);

        record.apply(Action::BufferCreateEmpty(ActionBufferCreateEmpty::new("name", 100, 0)))?;
        println!("Before:\n{}\n\n", serde_yaml::to_string(&record).unwrap());
        record.undo()?;
        // assert_eq!("name", record.target().name);
        println!("After:\n{}\n\n", serde_yaml::to_string(&record).unwrap());

        record.redo()?;
        println!("After redo:\n{}\n\n", serde_yaml::to_string(&record).unwrap());
        // record.redo()?;
        // assert_eq!("newname", record.target().name);

        Ok(())
    }
}
