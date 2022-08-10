// SPDX-License-Identifier: WTFPL
use crate::device::Device;
use std::{ffi::OsStr, fs, io, os::unix::prelude::*, path::Path};

mod sensor;
mod sensor_type;

const BAD_PATH: &str = "internal error: bad path";

fn module(path: &Path) -> Result<Device, io::Error> {
    let name_path = path.join("name");
    let module_name = match fs::read_to_string(name_path) {
        Err(e) => {
            eprintln!("can't read kmod name: {}", e);
            path.file_name()
                .and_then(OsStr::to_str)
                .expect(BAD_PATH)
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
    Ok(Device {
        sources: sensors,
        name: module_name,
        path: path.to_owned(),
    })
}

pub fn modules() -> Result<Vec<Device>, io::Error> {
    let mut modules = Vec::new();
    // usually called once per program instance
    let name_rx = regex::bytes::Regex::new(r"hwmon(\d+)$").unwrap();
    for dent in fs::read_dir("/sys/class/hwmon/")? {
        let dent = dent?;
        // if name_rx.is_match(dent.file_name().as_bytes()) {
        if let Some(caps) = name_rx.captures(dent.file_name().as_bytes()) {
            let idx = unsafe { std::str::from_utf8_unchecked(&caps[1]) }
                .parse::<i32>()
                .expect("not a number");
            modules.push((idx, module(&dent.path())?));
        }
        // }
    }
    modules.sort_by(|(i, _), (j, _)| i.cmp(j));
    Ok(modules.into_iter().map(|(_, m)| m).collect())
}
