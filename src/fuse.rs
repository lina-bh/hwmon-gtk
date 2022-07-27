use std::cell::Cell;

// SPDX-License-Identifier: WTFPL
#[derive(Debug)]
pub struct Fuse(Cell<bool>);

impl Fuse {
    pub fn new() -> Fuse {
        Fuse(Cell::new(false))
    }

    pub fn blow(&self) {
        self.0.set(true);
    }

    pub fn blown(&self) -> bool {
        self.0.get()
    }
}
