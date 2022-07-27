// SPDX-License-Identifier: WTFPL
use std::{
    fmt::{Debug, Display},
    fs::File,
    io,
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
    str::{FromStr, Utf8Error},
};

pub trait Source: Send {
    fn read(&self) -> Option<f32>;
    fn unit(&self) -> &str;
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub enum ReadIntoErrorKind<F>
where
    F: FromStr,
{
    Io(io::Error),
    Utf8(Utf8Error, Vec<u8>),
    Parse(F::Err, String),
}

#[derive(Debug)]
pub struct ReadIntoError<F: FromStr>
where
    F: FromStr,
    F::Err: Debug, // F::Err: std::error::Error,
{
    path: Option<PathBuf>,
    kind: ReadIntoErrorKind<F>,
}

impl<F> ReadIntoError<F>
where
    F: FromStr,
    F::Err: Debug,
{
    fn new<P: AsRef<Path>>(path: Option<P>, kind: ReadIntoErrorKind<F>) -> ReadIntoError<F> {
        ReadIntoError {
            path: path.map(|p| p.as_ref().to_owned()),
            kind,
        }
    }
}

impl<F: FromStr> Display for ReadIntoError<F>
where
    F::Err: std::error::Error + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ReadIntoErrorKind::*;
        match &self.kind {
            Io(e) => match &self.path {
                None => write!(f, "couldn't read file: {}", e),
                Some(p) => write!(f, "couldn't read {:?}: {}", p, e),
            },
            Utf8(e, b) => write!(f, "non-utf8 output b\"{:?}\": {}", b, e),
            Parse(e, s) => write!(f, "unexpected output \"{}\": {}", s, e),
        }
    }
}

impl<F> std::error::Error for ReadIntoError<F>
where
    F: FromStr + Debug,
    F::Err: std::error::Error,
{
}

pub fn read_into<I, P>(f: &File, p: Option<P>) -> Result<I, ReadIntoError<I>>
where
    File: FileExt,
    I: FromStr,
    I::Err: Debug,
    P: AsRef<Path>,
{
    let mut buf: [u8; 16] = [0; 16];
    if let Err(e) = f.read_at(&mut buf, 0) {
        return Err(ReadIntoError::new(p, ReadIntoErrorKind::Io(e)));
    };
    let s = match std::str::from_utf8(&buf) {
        Err(e) => {
            return Err(ReadIntoError::new(
                p,
                ReadIntoErrorKind::Utf8(e, buf.to_vec()),
            ))
        }
        Ok(s) => s,
    };
    let v = match s.trim_matches(|c| c == '\0' || c == '\n').parse::<I>() {
        Err(e) => {
            return Err(ReadIntoError::new(
                p,
                ReadIntoErrorKind::Parse(e, s.to_owned()),
            ));
        }
        Ok(v) => v,
    };
    Ok(v)
}
