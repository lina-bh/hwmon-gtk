// SPDX-License-Identifier: WTFPL
use once_cell::sync::Lazy;
use regex::bytes::Regex as BytesRegex;

use std::fs::{self, File};
use std::io;
use std::os::unix::ffi::OsStrExt;

use std::path::{Path, PathBuf};

use super::sensor_type::SensorType;
use crate::fuse::Fuse;
use crate::source::{read_into, Source};

#[derive(Debug)]
pub struct Sensor {
    input: File,
    label: String,
    typ: SensorType,
    path: PathBuf,
    fail: Fuse,
}

static NAME_RX: Lazy<BytesRegex> =
    Lazy::new(|| BytesRegex::new(r"(.+)(\d+)_").expect("malformed regex"));

impl Sensor {
    fn new(path: &Path) -> Result<Sensor, io::Error> {
        let name = path.file_name().expect("bad path from Module::new");
        let caps = NAME_RX
            .captures(name.as_bytes())
            .expect("bad path from Module::new");
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
            fail: Fuse::new(),
        })
    }
}

impl Source for Sensor {
    fn read(&self) -> Option<f32> {
        if self.fail.blown() {
            return None;
        }

        match read_into::<i32, _>(&self.input, Some(self.path.as_path())) {
            Ok(v) => Some(match self.typ {
                SensorType::Fan => v as f32,
                _ => v as f32 / 1000.0,
            }),
            Err(e) => {
                self.fail.blow();
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

pub fn sensors<P: AsRef<Path>>(path: P) -> Result<Vec<Box<dyn Source>>, io::Error> {
    let mut sensors = Vec::new();
    let name_rx = BytesRegex::new(r"(.+)(\d+)_input$").expect("malformed regex");
    for dent in fs::read_dir(&path)? {
        let dent = dent?;
        if !name_rx.is_match(dent.file_name().as_bytes()) {
            continue;
        };
        sensors.push(Box::new(Sensor::new(&dent.path())?) as Box<dyn Source>);
    }
    sensors.sort_by(|l, r| l.name().cmp(r.name()));
    Ok(sensors)
}
