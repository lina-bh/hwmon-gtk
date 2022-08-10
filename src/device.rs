use std::{fs, path::PathBuf};

// SPDX-License-Identifier: WTFPL
use crate::source::{Input, Source};

pub struct Device {
    pub name: String,
    pub sources: Vec<Source>,
    pub path: PathBuf,
}
