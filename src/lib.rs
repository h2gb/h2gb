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
    use redo::Record;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_action() -> SimpleResult<()> {
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

// record.apply(Add('a'))?;
// record.apply(Add('b'))?;
// record.apply(Add('c'))?;
// assert_eq!(record.target(), "abc");
// record.undo()?;
// record.undo()?;
// record.undo()?;
// assert_eq!(record.target(), "");
// record.redo()?;
// record.redo()?;
// record.redo()?;
// assert_eq!(record.target(), "abc");


        Ok(())
    }
}
