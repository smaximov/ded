use std::cmp::{Ordering};
use std::error;
use std::fmt;
use std::ops::{Deref};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::result;

use sequence_trie::{SequenceTrie};

use util::{sha1};

#[derive(Clone, Debug)]
pub struct Entry {
    path: PathBuf,
    hash: String
}

impl Entry {
    pub fn new(path: PathBuf) -> Self {
        let hash = sha1(&path.to_string_lossy());
        Entry::with_hash(hash, path)
    }

    pub fn with_hash(hash: String, path: PathBuf) -> Self {
        Entry {
            path: path,
            hash: hash
        }
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn hash_short(&self, width: usize) -> &str {
        &self.hash()[0..width]
    }

    pub fn basename(&self) -> String {
        let mut basename = self.path.file_name()
            .unwrap_or_else(|| self.path.as_os_str())
            .to_string_lossy().into_owned();

        if self.path.is_dir() {
            basename.push(MAIN_SEPARATOR);
        }

        basename
    }

    pub fn is_hidden(&self) -> bool {
        self.basename().starts_with(".")
    }

    pub fn path(&self) -> &Path {
        self.path.deref()
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Entry {
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let this = &self.path;
        let that = &other.path;

        let same_type = this.is_dir() == that.is_dir() ||
            this.is_file() == that.is_file();

        let this_hidden = self.is_hidden();
        let that_hidden = other.is_hidden();

        if this_hidden != that_hidden {
            return that_hidden.partial_cmp(&this_hidden);
        }

        if !same_type {
            return that.is_dir()
                .partial_cmp(&this.is_dir())
        }

        this.partial_cmp(that)
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug)]
pub enum Error {
    Absent(String),
    Ambiguous(String, Vec<String>)
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Absent(ref hash) =>
                write!(fmt, "Cannot find entry with hash prefix {}", hash),
            Error::Ambiguous(ref hash, ref matches) =>
                write!(fmt, "Ambiguous hash prefix {}, matches {}", hash, matches.join(", "))
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Absent(_) => "Cannot find hash",
            Error::Ambiguous(..) => "Ambiguous prefix hash"
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct EntryMap(SequenceTrie<char, Entry>);

impl EntryMap {
    pub fn new() -> Self {
        EntryMap(SequenceTrie::new())
    }

    pub fn from_vec(vec: Vec<Entry>) -> Self {
        let mut map = Self::new();
        for entry in vec {
            map.insert(entry);
        }
        map
    }

    pub fn insert(&mut self, entry: Entry) -> bool {
        let hash = String::from(entry.hash());
        let hash_chars: Vec<_> = hash.chars().collect();
        self.0.insert(&hash_chars, entry)
    }

    pub fn get_all(&self, hash: &str) -> Vec<&Entry> {
        let hash_chars: Vec<_> = hash.chars().collect();
        let node = self.0.get_node(&hash_chars);
        node.map_or(Vec::new(), |node| {
            node.values().collect()
        })
    }

    pub fn get(&self, hash: &str) -> Result<&Entry> {
        let entries = self.get_all(hash);
        match entries.len() {
            0 => Err(Error::Absent(String::from(hash))),
            1 => Ok(entries[0]),
            _ => {
                let matches: Vec<String> = entries.iter()
                    .map(|e| String::from(e.hash()))
                    .collect();

                Err(Error::Ambiguous(String::from(hash), matches))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{PathBuf};

    #[test]
    fn get_all() {
        let foo = Entry::with_hash(String::from("foo"), PathBuf::from("/foo"));
        let foobar = Entry::with_hash(String::from("foobar"), PathBuf::from("/foo/bar"));
        let bar = Entry::with_hash(String::from("bar"), PathBuf::from("/bar"));

        let mut map = EntryMap::new();

        map.insert(foo.clone());
        map.insert(foobar.clone());
        map.insert(bar.clone());

        let mut expected;
        let mut result;

        expected = vec![&foo, &foobar];
        result = map.get_all("foo");
        expected.sort();
        result.sort();
        assert_eq!(expected, result);

        expected = vec![&foobar];
        result = map.get_all("foobar");
        expected.sort();
        result.sort();
        assert_eq!(expected, result);

        expected = vec![&bar];
        result = map.get_all("bar");
        expected.sort();
        result.sort();
        assert_eq!(expected, result);

        expected = vec![&foo, &bar, &foobar];
        result = map.get_all("");
        expected.sort();
        result.sort();
        assert_eq!(expected, result);
    }

    #[test]
    fn get() {
        let mut map = EntryMap::new();
        let entry = Entry::new(PathBuf::from("/root"));
        let hash_short = entry.hash_short(8);

        map.insert(entry.clone());

        assert!(map.get(&hash_short).is_ok());
        assert!(map.get("derp").is_err());

        map.insert(Entry::with_hash(String::from(hash_short), PathBuf::from("/root/foo")));

        assert!(map.get(&hash_short).is_err());
    }
}
