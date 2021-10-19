use std::fs;
use std::iter;
use std::path::{Path, PathBuf};

use super::values::{take_sub_value_at_address, value_from_file};
use super::DataResolverError;

enum Level {
    Dir,
    File,
}

pub struct DataPath<'a> {
    level: Level,
    path: PathBuf,
    address: &'a [&'a str],
}

impl<'a> DataPath<'a> {
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
    pub fn files(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        match fs::read_dir(&self.path) {
            Ok(reader) => Box::new(
                reader
                    .filter_map(|dir_entry| dir_entry.ok())
                    .map(|dir_entry| dir_entry.path()),
            ),
            _ => Box::new(iter::empty::<PathBuf>()),
        }
    }
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
    fn get_value(&self, path: &Path) -> Result<serde_yaml::Value, DataResolverError> {
        let mut value = value_from_file(path)?;
        take_sub_value_at_address(&mut value, self.address)
    }
    fn index(&self) -> PathBuf {
        self.path.join("index.yml")
    }
    pub fn join<P: AsRef<Path>>(&self, tail: P) -> Self {
        Self {
            level: Level::File,
            path: self.path.join(tail),
            address: self.address,
        }
    }
    pub fn new<P: Into<PathBuf>>(path: P, address: &'a [&'a str]) -> Self {
        Self {
            address,
            level: Level::Dir,
            path: path.into(),
        }
    }
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
                    self.path.push(head);
                    self.address = tail;
                    self.level = File;
                }
            }
        }
        Some(self)
    }
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
