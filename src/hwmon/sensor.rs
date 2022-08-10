// SPDX-License-Identifier: WTFPL
use super::{sensor_type::SensorType, BAD_PATH};
use crate::source::{read_into, Input, Source};
use once_cell::sync::Lazy;
use std::{
    cell::Cell,
    fs::{self, File},
    io,
    os::unix::prelude::*,
    path::{Path, PathBuf},
};

static NAME_RX: Lazy<regex::bytes::Regex> =
    Lazy::new(|| regex::bytes::Regex::new(r"(.+)(\d+)_(.+)").expect("malformed regex"));

#[derive(Debug)]
pub struct Sensor {
    input: File,
    label: String,
    typ: SensorType,
    pub path: PathBuf,
    fail: Cell<bool>,
}

impl Sensor {
    fn new(path: &Path) -> Result<Sensor, io::Error> {
        // assert that super::modules passed correct path
        let name = path.file_name().expect(BAD_PATH);
        let caps = NAME_RX.captures(name.as_bytes()).expect(BAD_PATH);
        let input = File::open(&path)?;
        // fairly certain that `name` is valid ascii/utf8 because we already passed the regex
        let typ = unsafe { std::str::from_utf8_unchecked(&caps[1]) };
        let idx = unsafe { std::str::from_utf8_unchecked(&caps[2]) }
            .parse::<u16>()
            .unwrap();
        let sensor_name = format!("{}{}", typ, idx);
        let mut label_path = path.to_owned();
        label_path.pop();
        label_path.push(format!("{}_label", sensor_name));
        let label = match fs::read_to_string(label_path) {
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => sensor_name,
                _ => return Err(e),
            },
            Ok(mut s) => {
                if s.ends_with('\n') {
                    s.truncate(s.len() - 1);
                }
                s
            }
        };
        Ok(Sensor {
            input,
            typ: SensorType::from_str(typ),
            label,
            path: path.to_owned(),
            fail: Cell::new(false),
        })
    }
}

impl Input for Sensor {
    fn read(&self) -> Option<f32> {
        if self.fail.get() {
            return None;
        }

        match read_into::<i32>(&self.input, self.path.as_path()) {
            Ok(v) => Some(match self.typ {
                SensorType::Fan => v as f32,
                _ => v as f32 / 1000.0,
            }),
            Err(e) => {
                self.fail.set(true);
                eprintln!("{}", e);
                None
            }
        }
    }

    fn unit(&self) -> &str {
        self.typ.unit()
    }

    fn name(&self) -> &str {
        &self.label
    }
}

pub fn sensors(path: &Path) -> Result<Vec<Source>, io::Error> {
    let mut sensors = Vec::new();
    for dent in fs::read_dir(path)? {
        let dent = dent?;
        if let Some(caps) = NAME_RX.captures(dent.file_name().as_bytes()) {
            if caps[3].eq(b"input") {
                sensors.push(Box::new(Sensor::new(&dent.path())?));
            }
        }
    }
    sensors.sort_by(|l, r| l.path.file_name().cmp(&r.path.file_name()));
    Ok(sensors.into_iter().map(Source::new).collect())
}
