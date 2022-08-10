// SPDX-License-Identifier: WTFPL
mod read_into;

pub use read_into::*;

type Output = f32;

pub trait Input: Send {
    fn read(&self) -> Option<Output>;
    fn unit(&self) -> &str;
    fn name(&self) -> &str;
}

pub struct Source {
    pub input: Box<dyn Input>,
    pub cur: Output,
    pub min: Output,
    pub max: Output,
    pub err: bool,
}

impl Source {
    pub fn new<T: Input + 'static>(input: Box<T>) -> Self {
        Self {
            input,
            cur: 0.,
            min: 0.,
            max: 0.,
            err: false,
        }
    }

    pub fn update(&mut self) {
        if let Some(v) = self.input.read() {
            self.cur = v;
            if self.min > v {
                self.min = v;
            }
            if self.max < v {
                self.max = v;
            }
        }
    }
}
