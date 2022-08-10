// SPDX-License-Identifier: WTFPL
use self::core::Core;
use crate::group::Group;
use crate::source::Source;
use std::{fs, io, os::unix::prelude::*};

mod core;

pub fn cores() -> Result<Group, io::Error> {
    let mut cores = Vec::new();
    let mut idx = 0;
    let policy_rx = regex::bytes::Regex::new(r"policy(\d+)$").unwrap();
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
