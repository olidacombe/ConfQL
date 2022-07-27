//! Data path representations.
//!
//! When resolving data from a collection of files, we iterate
//! down from top-level, and descend a [DataPath] until exhaustion,
//! merging values as we go.  That way, more specific data paths
//! override fields as necessary, and we can be flexible about how
//! the data for a yaml mapping is broken up into smaller files.
//!
//! At a high-level, given a data address of ["a", "b", "c"], we can iterate like so:
//!
//! - look for `a.b.c` in `index.yml`
//! - look for `b.c` in `a.yml`
//! - look for `b.c` in `a/index.yml`
//! - look for `c` in `a/b.yml`
//! - look for `c` in `a/b/index.yml`
//! - merge all data from `a/b/c.yml`
//! - merge all data from `a/b/c/index.yml`
//!
//! [DataPath] provides a simple means for performing this process.
use super::filters::{FilterMap, Filters};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use super::values::{take_sub_value_at_address, value_from_file};
use super::DataResolverError;

enum Level {
    Dir,
    File,
}

/// Represents a position in the data directory when resolving data.
pub struct DataPath<'a> {
    level: Level,
    path: PathBuf,
    address: &'a [&'a str],
    filters: Filters<'a>,
}

impl<'a> DataPath<'a> {
    /// Takes self by value, and steps to the next logical data path (mutating self).  Returns None
    /// if there's nowhere to go.
    pub fn descend(mut self) -> Option<Self> {
        use Level::{Dir, File};
        match &self.level {
            File => {
                if !self.path.is_dir() {
                    return None;
                }
                self.level = Dir;
            }
            Dir => {
                if let Some((head, tail)) = self.address.split_first() {
                    self.filters.descend(&head);
                    self.path.push(head);
                    self.address = tail;
                    self.level = File;
                }
            }
        }
        Some(self)
    }
    /// Returns whether or not this instance is exhausted, i.e. when [descend](DataPath::descend()) would
    /// be a no-op
    pub fn done(&self) -> bool {
        use Level::{Dir, File};
        match &self.level {
            File => false,
            Dir => self.address.is_empty(),
        }
    }
    fn file(&self) -> PathBuf {
        self.path.with_extension("yml")
    }
    /// Returns the current path file stem (i.e. basename without file extension)
    pub fn file_stem(&self) -> Option<&OsStr> {
        self.path.file_stem()
    }
    fn get_value(&self, path: &Path) -> Result<serde_yaml::Value, DataResolverError> {
        let mut value = value_from_file(path)?;
        take_sub_value_at_address(&mut value, self.address)
    }
    fn index(&self) -> PathBuf {
        self.path.join("index.yml")
    }
    /// Spawns a new instance with a given path suffix appended, and same data address.
    pub fn join<P: AsRef<Path>>(&self, tail: P) -> Self {
        Self {
            level: Level::File,
            path: self.path.join(tail),
            address: self.address,
            filters: self.filters.clone(),
        }
    }
    /// Creates a new instance from a path and data address.
    pub fn new<P: Into<PathBuf>>(path: P, address: &'a [&'a str]) -> Self {
        Self {
            address,
            level: Level::Dir,
            path: path.into(),
            filters: Filters::new(),
        }
    }
    /// Creates a vector of new instances, one for each file/directory at the current path.
    pub fn sub_paths(&self) -> Vec<Self> {
        fs::read_dir(&self.path).map_or_else(
            |_| vec![],
            |reader| {
                reader
                    .filter_map(|dir_entry| dir_entry.ok())
                    .map(|dir_entry| dir_entry.file_name())
                    .map(|p| self.join(p))
                    .collect()
            },
        )
    }
    /// Tries to convert the current position to a [serde_yaml::Value].
    pub fn value(&self) -> Result<serde_yaml::Value, DataResolverError> {
        match &self.level {
            Level::Dir => self.get_value(&self.index()),
            Level::File => self.get_value(&self.file()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use indoc::indoc;
    use test_files::TestFiles;
    use test_utils::yaml;

    trait GetDataPath<'a> {
        fn data_path(&self, address: &'a [&'a str]) -> DataPath<'a>;
    }

    impl<'a> GetDataPath<'a> for TestFiles {
        fn data_path(&self, address: &'a [&'a str]) -> DataPath<'a> {
            DataPath::new(self.path(), address)
        }
    }

    #[test]
    fn resolves_num_at_root() -> Result<()> {
        let mocks = TestFiles::new();
        mocks.file(
            "index.yml",
            indoc! {"
                ---
                3
            "},
        );
        let v = mocks.data_path(&[]).value()?;
        assert_eq!(v, yaml! {"3"});
        Ok(())
    }

    #[test]
    fn resolves_num_deeper() -> Result<()> {
        let mocks = TestFiles::new();
        mocks.file(
            "index.yml",
            indoc! {"
	            ---
	            a:
	                b:
	                    c: 3
	        "},
        );
        let v = mocks.data_path(&["a", "b", "c"]).value()?;
        assert_eq!(v, yaml! {"3"});
        Ok(())
    }

    #[test]
    fn resolves_list_num_at_index() -> Result<()> {
        let mocks = TestFiles::new();
        mocks.file(
            "index.yml",
            indoc! {"
	            ---
	            a:
	            - 4
	            - 5
	            - 6
	        "},
        );
        let v = mocks.data_path(&["a"]).value()?;
        assert_eq!(v, yaml! {"[4, 5, 6]"});
        Ok(())
    }
}
