use std::fs::{File, DirBuilder};
use std::path::Path;
use std::io::prelude::*;
use std::io;
use std::fmt;
use std::error;

static CACHE_DIR: &'static str = "cache";

macro_rules! prefix {
    ($path:expr) => (Path::new(CACHE_DIR).join($path));
}

#[derive(Debug)]
pub enum CacheError {
    Io(io::Error),
    Callback(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CacheError::Io(ref err) => write!(f, "{}", err),
            CacheError::Callback(ref message) => write!(f, "{}", message),
        }
    }
}

impl From<io::Error> for CacheError {
    fn from(err: io::Error) -> CacheError {
        CacheError::Io(err)
    }
}

impl From<String> for CacheError {
    fn from(err: String) -> CacheError {
        CacheError::Callback(err)
    }
}

impl error::Error for CacheError {
    fn description(&self) -> &str {
        match *self {
            CacheError::Io(ref err) => err.description(),
            CacheError::Callback(ref msg) => msg,
        }
    }
}

pub type CacheResult = Result<String, CacheError>;

pub fn get_cached_or_compute<E, F>(key: &str, fun: F) -> CacheResult
        where E: fmt::Display,
              F: Fn(&str) -> Result<String, E> {
    let root = Path::new(CACHE_DIR);
    try!(DirBuilder::new()
        .recursive(true)
        .create(&root));

    let path = root.join(key);
    if let Ok(mut file) = File::open(&path) {
        let mut buf = String::new();
        if let Ok(_) = file.read_to_string(&mut buf) {
            return Ok(buf)
        }
    }

    let value = match fun(key) {
        Ok(value) => value,
        Err(err) => return Err(CacheError::Callback(err.to_string())),
    };

    let mut file = try!(File::create(&path));
    try!(file.write_all(value.as_bytes()));

    Ok(value)
}

#[test]
fn get_existing() {
    let existing_key = "YEP";
    let path = prefix!(existing_key);

    File::create(path).unwrap();

    let response = get_cached_or_compute(
        existing_key,
        |_| Err(io::Error::new(io::ErrorKind::Interrupted, "Fuu")));

    assert!(response.is_ok());
}
