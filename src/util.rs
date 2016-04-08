use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crypto::digest::{Digest};
use crypto::sha1::{Sha1};

pub fn sha1(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(input);
    hasher.result_str()
}

pub fn temp_dir<P: AsRef<Path>>(prefix: P) -> io::Result<PathBuf> {
    let mut temp_dir = env::temp_dir();
    temp_dir.push(prefix);
    try!(fs::create_dir_all(&temp_dir));
    Ok(temp_dir)
}

pub fn width(n: usize) -> usize {
    (n as f64).log10().floor() as usize + 1
}

#[cfg(test)]
mod tests {
    #[test]
    fn width() {
        assert_eq!(1, super::width(1));
        assert_eq!(1, super::width(9));
        assert_eq!(2, super::width(10));
        assert_eq!(2, super::width(99));
        assert_eq!(3, super::width(100));
    }
}
