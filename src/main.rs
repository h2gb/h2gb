use redo::{Command, Record};
use serde::{Serialize, Deserialize};

mod lib;

#[derive(Serialize, Deserialize, Debug)]
struct MyTarget {
    i: u32,
    c: char,
    s: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddAction {
    i: u32,
    old: Option<u32>,
}

impl AddAction {
    fn new(i: u32) -> Self {
        return AddAction {
            i: i,
            old: None
        }
    }
}

impl Command for AddAction {
    type Target = MyTarget;
    type Error = &'static str;

    fn apply(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        self.old = Some(t.i);
        t.i += self.i;
        Ok(())
    }

    fn undo(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        t.i = self.old.unwrap();
        self.old = None;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SubtractAction {
    i: u32,
    old: Option<u32>,
}

impl SubtractAction {
    fn new(i: u32) -> Self {
        return SubtractAction {
            i: i,
            old: None
        }
    }
}

impl Command for SubtractAction {
    type Target = MyTarget;
    type Error = &'static str;

    fn apply(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        self.old = Some(t.i);
        t.i -= self.i;
        Ok(())
    }

    fn undo(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        t.i = self.old.unwrap();
        self.old = None;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ReplaceAction {
    c: char,
    old: Option<char>,
}

impl ReplaceAction {
    fn new(c: char) -> Self {
        return ReplaceAction {
            c: c,
            old: None,
        }
    }
}

impl Command for ReplaceAction {
    type Target = MyTarget;
    type Error = &'static str;

    fn apply(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        self.old = Some(t.c);
        t.c = self.c;

        Ok(())
    }

    fn undo(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        t.c = self.old.unwrap();
        self.old = None;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Action {
    Add(AddAction),
    Subtract(SubtractAction),
    Replace(ReplaceAction),
}

impl Command for Action {
    type Target = MyTarget;
    type Error = &'static str;

    fn apply(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        match self {
            Action::Add(a) => a.apply(t),
            Action::Subtract(a) => a.apply(t),
            Action::Replace(a) => a.apply(t),
        }
    }

    fn undo(&mut self, t: &mut MyTarget) -> redo::Result<Self> {
        match self {
            Action::Add(a) => a.undo(t),
            Action::Subtract(a) => a.undo(t),
            Action::Replace(a) => a.undo(t),
        }
    }
}

fn try_redo() -> redo::Result<Action> {
    let record = MyTarget {
        i: 0,
        c: 'A',
        s: String::from(""),
    };

    let mut record = Record::new(record);

    println!("Original: {:?}", record.target());

    record.apply(Action::Add(AddAction::new(3))).unwrap();
    println!("After adding 3: {:?} (expected: i=3)", record.target());
    assert!(record.target().i == 3);

    record.apply(Action::Subtract(SubtractAction::new(2))).unwrap();
    println!("After subtracting 2: {:?} (expected: i=1)", record.target());
    assert!(record.target().i == 1);

    record.apply(Action::Add(AddAction::new(1))).unwrap();
    println!("After adding 1: {:?} (expected: i=2)", record.target());
    assert!(record.target().i == 2);

    record.undo()?;
    println!("After undo: {:?} (expected: i=1)", record.target());
    assert!(record.target().i == 1);

    record.redo()?;
    println!("After redo: {:?} (expected: i=2)", record.target());
    assert!(record.target().i == 2);

    record.undo()?;
    println!("After undo: {:?} (expected: i=1)", record.target());
    assert!(record.target().i == 1);

    record.undo()?;
    println!("After undo: {:?} (expected: i=3)", record.target());
    assert!(record.target().i == 3);

    println!("JSON: {}", serde_json::to_string(&record).unwrap());
    println!();
    println!("YAML: {}", serde_yaml::to_string(&record).unwrap());
    println!();
    println!("RON: {}", ron::ser::to_string(&record).unwrap());
    let s = ron::ser::to_string(&record).unwrap();
    println!("Serialized: {}", s);

    let mut record: redo::record::Record<Action> = ron::de::from_str(&s).unwrap();
    println!("After deserialize: {:?} (expected: i=3)", record.target());
    assert!(record.target().i == 3);

    record.undo()?;
    println!("After undo: {:?} (expected: i=0)", record.target());
    assert!(record.target().i == 0);

    record.redo()?;
    println!("After redo: {:?} (expected: i=3)", record.target());
    assert!(record.target().i == 3);

    record.redo()?;
    println!("After redo: {:?} (expected: i=1)", record.target());
    assert!(record.target().i == 1);

    record.redo()?;
    println!("After redo: {:?} (expected: i=2)", record.target());
    assert!(record.target().i == 2);

    record.apply(Action::Replace(ReplaceAction::new('c'))).unwrap();
    println!("After replace: {:?} (expected: c=c)", record.target());
    assert!(record.target().c == 'c');

    record.apply(Action::Replace(ReplaceAction::new('d'))).unwrap();
    println!("After replace: {:?} (expected: c=d)", record.target());
    assert!(record.target().c == 'd');

    record.undo()?;
    println!("After undo: {:?} (expected: c=c)", record.target());
    assert!(record.target().c == 'c');

    record.undo()?;
    println!("After undo: {:?} (expected: c=A)", record.target());
    assert!(record.target().c == 'A');

    let s = ron::ser::to_string(&record).unwrap();
    println!("Serialized: {}", s);

    let mut record: redo::record::Record<Action> = ron::de::from_str(&s).unwrap();
    println!("After deserialize: {:?} (expected: c=A)", record.target());
    assert!(record.target().c == 'A');

    record.redo()?;
    println!("After redo: {:?} (expected: c=c)", record.target());
    assert!(record.target().c == 'c');

    record.apply(Action::Replace(ReplaceAction::new('e'))).unwrap();
    println!("After replace: {:?} (expected: c=e)", record.target());
    assert!(record.target().c == 'e');

    record.redo()?;
    println!("After redo: {:?} (expected: c=e)", record.target());
    assert!(record.target().c == 'e');

    Ok(())
}

fn main() {
    try_redo().unwrap();
    let a: lib::H2Project = Default::default();
    let s = ron::ser::to_string(&a).unwrap();
    println!("Serialized: {}", s);
}
