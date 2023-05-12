use std::error::Error;
use std::sync::Arc;
use std::{fmt, io};

#[derive(Debug)]
pub enum SystemFontError {
    Io(io::Error),
    MemoryUnwrap(Arc<Vec<u8>>),
    NotScalable,
}

impl Error for SystemFontError {}

impl fmt::Display for SystemFontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error when loading font: {:?}", e),
            Self::MemoryUnwrap(_) => write!(f, "Memory error, couldn't reap Arc<…>"),
            Self::NotScalable => write!(f, "Font not scalable!"),
        }
    }
}

impl From<io::Error> for SystemFontError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<Arc<Vec<u8>>> for SystemFontError {
    fn from(value: Arc<Vec<u8>>) -> Self {
        Self::MemoryUnwrap(value)
    }
}
