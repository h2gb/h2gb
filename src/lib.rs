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
    use multi_vector::MultiVector;

    use h2project::H2Project;
    use action::Action;
    use action::null::NullAction;
    use action::change_project_name::ActionChangeProjectName;
    use redo::Record;

    #[test]
    fn test_vector() -> SimpleResult<()> {
        let mut mv: MultiVector<u32> = MultiVector::new();
        mv.create_vector("test", 1000)?;

        let s = ron::ser::to_string(&mv).unwrap();
        println!("Serialized: {}", s);

        Ok(())
    }

    #[test]
    fn test_change_project_name() {
        let project = H2Project::new("name", "version");

        let mut record: Record<Action> = Record::new(project);

        println!("Start: {}", record.target());
        record.apply(Action::Null(NullAction::new())).unwrap();
        record.apply(Action::Null(NullAction::new())).unwrap();
        record.apply(Action::Null(NullAction::new())).unwrap();
        record.apply(Action::ChangeProjectName(ActionChangeProjectName::new("Hi"))).unwrap();
        println!("After changing name to 'hi': {}", record.target());
        record.undo().unwrap();
        println!("After undo: {}", record.target());
        record.redo().unwrap();
        println!("After redo: {}", record.target());
    }
}
