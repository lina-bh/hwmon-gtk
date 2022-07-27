// SPDX-License-Identifier: WTFPL
use crate::source::Source;

pub struct Group {
    pub name: String,
    pub sources: Vec<Box<dyn Source>>,
}
