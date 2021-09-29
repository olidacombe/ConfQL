use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

mod values;
use values::{get_sub_value_at_address, value_from_file};

#[derive(Error, Debug)]
pub enum DataResolverError {
    #[error("Key `{0}` not found")]
    KeyNotFound(String),
    #[error("Data not found")]
    DataNotFound,
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub struct DataResolver<'a> {
    root: &'a Path,
}

impl<'a> DataResolver<'a> {
    pub fn get_non_nullable<T>(&self, address: &[&str]) -> Result<T, DataResolverError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.get_nullable(address)?
            .ok_or(DataResolverError::DataNotFound)
    }

    pub fn get_non_nullable_list<T>(&self, address: &[&str]) -> Result<Vec<T>, DataResolverError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.get_nullable_list(address)?
            .ok_or(DataResolverError::DataNotFound)
    }

    pub fn get_nullable<T>(&self, address: &[&str]) -> Result<Option<T>, DataResolverError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let path = self.root.join("index.yml");
        let value = value_from_file(&path)?;
        let value = get_sub_value_at_address(&value, address)?;
        let value = serde_yaml::from_value(value.to_owned())?;
        Ok(value)
    }

    pub fn get_nullable_list<T>(
        &self,
        address: &[&str],
    ) -> Result<Option<Vec<T>>, DataResolverError>
    where
        T: for<'de> Deserialize<'de>,
    {
        Err(DataResolverError::DataNotFound)
    }
}

impl<'a> From<&'a Path> for DataResolver<'a> {
    fn from(path: &'a Path) -> Self {
        Self { root: path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use indoc::indoc;
    use test_files::TestFiles;

    trait GetResolver {
        fn resolver(&self) -> DataResolver;
    }

    impl GetResolver for TestFiles {
        fn resolver(&self) -> DataResolver {
            DataResolver::from(self.path())
        }
    }

    #[test]
    fn resolves_non_nullable_int_at_root() -> Result<()> {
        let mocks = TestFiles::new().unwrap();
        mocks.file(
            "index.yml",
            indoc! {"
                ---
                version: 3
            "},
        )?;
        let i: u32 = mocks.resolver().get_non_nullable(&["version"])?;
        assert_eq!(i, 3);
        Ok(())
    }

    #[test]
    fn resolves_non_nullable_int_deeper() -> Result<()> {
        let mocks = TestFiles::new().unwrap();
        mocks.file(
            "index.yml",
            indoc! {"
                ---
                a:
                    b:
                        c: 3
            "},
        )?;
        let i: u32 = mocks.resolver().get_non_nullable(&["a", "b", "c"])?;
        assert_eq!(i, 3);
        Ok(())
    }

    #[test]
    fn resolves_non_nullable_int() -> Result<()> {
        let test_cases = [
            [
                "a/b/c.yml",
                indoc! {"
                    ---
                    3
                "},
            ],
            [
                "a/b/c/index.yml",
                indoc! {"
                    ---
                    3
                "},
            ],
            [
                "a/b/index.yml",
                indoc! {"
                    ---
                    c: 3
                "},
            ],
            [
                "a/b.yml",
                indoc! {"
                    ---
                    c: 3
                "},
            ],
            [
                "a.yml",
                indoc! {"
                    ---
                    b:
                        c: 3
                "},
            ],
            [
                "index.yml",
                indoc! {"
                    ---
                    a:
                        b:
                            c: 3
                "},
            ],
        ];

        for [file, content] in test_cases {
            let mocks = TestFiles::new().unwrap();
            mocks.file(file, content)?;
            let i: u32 = mocks.resolver().get_non_nullable(&["a", "b", "c"])?;
            assert_eq!(i, 3);
        }
        Ok(())
    }
}
