use common::entities::{Line, Position};
use leptos::{create_rw_signal, ReadSignal, RwSignal, SignalGetUntracked, SignalSet, SignalUpdate};

#[derive(Copy, Clone)]
pub struct LineSignal {
    set: RwSignal<Line>,
    grow: RwSignal<Position>,
    value: RwSignal<Line>,
}

#[derive(Copy, Clone)]
pub struct LineReadSignal {
    pub set: ReadSignal<Line>,
    pub grow: ReadSignal<Position>,
    pub value: ReadSignal<Line>,
}

impl LineSignal {
    pub fn new(line: Line) -> Self {
        Self {
            set: create_rw_signal(line.clone()),
            grow: create_rw_signal(Position { x: 0.0, y: 0.0 }),
            value: create_rw_signal(line),
        }
    }

    pub fn get_untracked(&self) -> Line {
        self.value.get_untracked()
    }

    pub fn read_only(&self) -> LineReadSignal {
        LineReadSignal {
            set: self.set.read_only(),
            grow: self.grow.read_only(),
            value: self.value.read_only(),
        }
    }

    pub fn set(&self, line: Line) {
        self.value.set(line.clone());
        self.set.set(line.clone());
    }

    pub fn grow(&self, position: Position) {
        self.grow.set(position.clone());
        self.value.update(|line| line.points.push(position));
    }
}
