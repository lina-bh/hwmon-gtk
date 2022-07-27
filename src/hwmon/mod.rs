// SPDX-License-Identifier: WTFPL
use regex::bytes::Regex as BytesRegex;
use std::ffi::OsStr;
use std::os::unix::prelude::*;
use std::path::Path;
use std::{fs, io};

pub mod sensor;
pub mod sensor_type;

use crate::group::Group;

fn module(path: &Path) -> Result<Group, io::Error> {
    let name_path = path.join("name");
    let module_name = match fs::read_to_string(name_path) {
        Err(e) => {
            eprintln!("can't read module name: {}", e);
            path.file_name()
                .and_then(OsStr::to_str)
                .expect("unexpected mangled path")
                .to_owned()
        }
        Ok(mut s) => {
            if s.ends_with('\n') {
                s.truncate(s.len() - 1);
            }
            s
        }
    };
    let sensors = sensor::sensors(path)?;
    Ok(Group {
        // path: path.to_owned(),
        sources: sensors,
        name: module_name,
    })
}

pub fn modules() -> Result<Vec<Group>, io::Error> {
    let mut modules = Vec::new();
    // usually called once per program instance
    let name_rx = BytesRegex::new(r"hwmon(\d+)$").unwrap();
    for dent in fs::read_dir("/sys/class/hwmon/")? {
        let dent = dent?;
        if name_rx.is_match(dent.file_name().as_bytes()) {
            modules.push(module(&dent.path())?);
        }
    }
    Ok(modules)
}
