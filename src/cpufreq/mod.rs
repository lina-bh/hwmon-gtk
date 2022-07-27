// SPDX-License-Identifier: WTFPL
use regex::bytes::Regex as BytesRegex;
use std::fs::{self};
use std::io;
use std::os::unix::prelude::*;

mod core;

use self::core::Core;
use crate::group::Group;
use crate::source::Source;

#[derive(Debug)]
struct Cores(Vec<Core>);

pub fn cores() -> Result<Group, io::Error> {
    let mut cores = Vec::new();
    let mut idx = 0;
    let policy_rx = BytesRegex::new(r"policy(\d+)$").unwrap();
    for dent in fs::read_dir("/sys/devices/system/cpu/cpufreq")? {
        let dent = dent?;
        if policy_rx.is_match(dent.file_name().as_bytes()) {
            cores.push(Box::new(Core::new(&dent.path(), idx)?) as Box<dyn Source>);
            idx += 1;
        }
    }
    Ok(Group {
        name: "CPU".to_owned(),
        sources: cores,
    })
}
