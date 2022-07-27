// SPDX-License-Identifier: WTFPL

use std::fs::File;
use std::io;
use std::os::unix::prelude::FileExt;
use std::path::Path;

use crate::source::Source;

#[derive(Debug)]
pub struct Core {
    idx: u16,
    input: File,
    label: String,
}

impl Core {
    pub fn new<P: AsRef<Path>>(path: P, idx: u16) -> Result<Core, io::Error> {
        let mut cur_freq_path = path.as_ref().to_owned();
        cur_freq_path.push("scaling_cur_frequency");
        let input = File::open(&cur_freq_path)?;
        Ok(Core {
            idx,
            input,
            label: format!("CPU {}", idx),
        })
    }
}

impl Source for Core {
    fn read(&self) -> Option<f32> {
        let mut buf = [0, 16];
        if let Err(e) = self.input.read_at(&mut buf, 0) {
            eprintln!("can't read cpu{} freq: {}", self.idx, e);
            return None;
        }
        let s = match std::str::from_utf8(&buf) {
            Err(e) => {
                eprintln!("nonutf8 output from cpu{} freq {:?} ({})", self.idx, buf, e);
                return None;
            }
            Ok(s) => s,
        };
        let v = match s.trim_end().parse::<i32>() {
            Err(e) => {
                eprintln!(
                    "unexpected output from cpu{} freq \"{}\" ({})",
                    self.idx, s, e
                );
                return None;
            }
            Ok(v) => v,
        };
        Some(v as f32 / 1000.0)
    }

    fn unit(&self) -> &str {
        " MHz"
    }

    fn name(&self) -> &str {
        &self.label
    }
}
