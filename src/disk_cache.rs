use std::fs::{File, DirBuilder};
use std::path::Path;
use std::io::prelude::*;
use std::io;
use std::fmt;

static CACHE_DIR: &'static str = "cache";

macro_rules! cache_path {
    ($path:expr) => (Path::new(CACHE_DIR).join($path));
    () => (Path::new(CACHE_DIR));
}

quick_error! {
    #[derive(Debug)]
    pub enum CacheError {
        Io(err: io::Error) {
            from()
        }
        Callback(err: String) {
            from()
        }
    }
}

pub type CacheResult = Result<String, CacheError>;

pub fn get_cached_or_compute<E, F>(key: &str, fun: F) -> CacheResult
    where E: fmt::Display,
          F: Fn(&str) -> Result<String, E>
{
    let root = cache_path!();
    try!(DirBuilder::new()
        .recursive(true)
        .create(&root));

    let path = cache_path!(key);
    if let Ok(mut file) = File::open(&path) {
        let mut buf = String::new();
        if let Ok(_) = file.read_to_string(&mut buf) {
            info!("Cache hit for {}", key);
            return Ok(buf);
        }
    }

    info!("Cache miss for {}", key);

    let value = match fun(key) {
        Ok(value) => value,
        Err(err) => return Err(CacheError::Callback(err.to_string())),
    };

    let mut file = try!(File::create(&path));
    try!(file.write_all(value.as_bytes()));

    info!("New value for {} stored to disk", key);

    Ok(value)
}

#[test]
fn get_existing() {
    let existing_key = "YEP";
    let path = cache_path!(existing_key);

    File::create(path).unwrap();

    let response = get_cached_or_compute(existing_key, |_| {
        Err(io::Error::new(io::ErrorKind::Interrupted, "Fuu"))
    });

    assert!(response.is_ok());
}
