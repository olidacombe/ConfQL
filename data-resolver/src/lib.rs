use juniper::ID;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

mod data_path;
pub use data_path::DataPath;
mod values;
pub use values::Merge;

#[derive(Error, Debug)]
pub enum DataResolverError {
    #[error("Incompatible merge `{dst:?}` <- `{src:?}`")]
    IncompatibleYamlMerge {
        src: serde_yaml::Value,
        dst: serde_yaml::Value,
    },
    #[error("Cannot merge into non-mapping `{0:?}`")]
    CannotMergeIntoNonMapping(serde_yaml::Value),
    #[error("Key `{0}` not found")]
    KeyNotFound(String),
    #[error("Data not found")]
    DataNotFound,
    #[error("Attempted to access data from empty DataPath")]
    EmptyDataPathAccess,
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub struct DataResolver {
    root: PathBuf,
}

impl DataResolver {
    pub fn get<T>(&self, address: &[&str]) -> Result<T, DataResolverError>
    where
        T: for<'de> Deserialize<'de>,
        T: ResolveValue,
    {
        let data_path = DataPath::new(&self.root, address);
        let value = T::resolve_value(data_path)?;
        Ok(serde_yaml::from_value(value)?)
    }
}

impl From<PathBuf> for DataResolver {
    fn from(root: PathBuf) -> Self {
        Self { root }
    }
}

pub trait ResolveValue {
    fn merge_properties(
        _value: &mut serde_yaml::Value,
        _data_path: &DataPath,
    ) -> Result<(), DataResolverError> {
        Ok(())
    }
    fn resolve_value(data_path: DataPath) -> Result<serde_yaml::Value, DataResolverError> {
        let mut value = data_path.value().unwrap_or(serde_yaml::Value::Null);
        if data_path.done() {
            Self::merge_properties(&mut value, &data_path)?;
        } else if let Some(data_path) = data_path.descend() {
            if let Ok(mergee) = Self::resolve_value(data_path) {
                value.merge(mergee)?;
            }
        }
        Ok(value)
    }
}

impl ResolveValue for bool {}
impl ResolveValue for f64 {}
impl ResolveValue for ID {}
impl ResolveValue for String {}
impl ResolveValue for i32 {}
impl<T: ResolveValue> ResolveValue for Option<T> {
    fn resolve_value(data_path: DataPath) -> Result<serde_yaml::Value, DataResolverError> {
        T::resolve_value(data_path).or(Ok(serde_yaml::Value::Null))
    }
}
impl<T: ResolveValue> ResolveValue for Vec<T> {
    fn merge_properties(
        value: &mut serde_yaml::Value,
        data_path: &DataPath,
    ) -> Result<(), DataResolverError> {
        value.merge(
            data_path
                .sub_paths()
                .into_iter()
                .filter_map(|dp| T::resolve_value(dp).ok())
                .collect(),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::values::Merge;
    use super::*;
    use color_eyre::Result;
    use indoc::indoc;
    use test_files::TestFiles;

    #[derive(Debug, Deserialize, PartialEq)]
    struct MyObj {
        id: i32,
        name: String,
    }

    impl ResolveValue for MyObj {
        fn merge_properties(
            value: &mut serde_yaml::Value,
            data_path: &DataPath,
        ) -> Result<(), DataResolverError> {
            if let Ok(id) = i32::resolve_value(data_path.join("id")) {
                value.merge_at("id", id)?;
            }
            if let Ok(name) = String::resolve_value(data_path.join("name")) {
                value.merge_at("name", name)?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
    struct MyOtherObj {
        id: i32,
        alias: String,
    }

    impl ResolveValue for MyOtherObj {
        fn merge_properties(
            value: &mut serde_yaml::Value,
            data_path: &DataPath,
        ) -> Result<(), DataResolverError> {
            if let Ok(id) = i32::resolve_value(data_path.join("id")) {
                value.merge_at("id", id)?;
            }
            if let Ok(alias) = String::resolve_value(data_path.join("alias")) {
                value.merge_at("alias", alias)?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Query {
        my_obj: MyObj,
        my_list: Vec<MyOtherObj>,
    }

    impl ResolveValue for Query {
        fn merge_properties(
            value: &mut serde_yaml::Value,
            data_path: &DataPath,
        ) -> Result<(), DataResolverError> {
            if let Ok(my_obj) = MyObj::resolve_value(data_path.join("my_obj")) {
                value.merge_at("my_obj", my_obj)?;
            }
            if let Ok(my_list) = Vec::<MyOtherObj>::resolve_value(data_path.join("my_list")) {
                value.merge_at("my_list", my_list)?;
            }
            Ok(())
        }
    }

    trait GetResolver<'a> {
        fn data_path(&self, address: &'a [&'a str]) -> DataPath<'a>;
        fn resolver(&self) -> DataResolver;
    }

    impl<'a> GetResolver<'a> for TestFiles {
        fn data_path(&self, address: &'a [&'a str]) -> DataPath<'a> {
            DataPath::new(self.path().to_path_buf(), address)
        }
        fn resolver(&self) -> DataResolver {
            DataResolver {
                root: self.path().to_path_buf(),
            }
        }
    }

    #[test]
    fn resolves_num() -> Result<()> {
        color_eyre::install()?;
        let mocks = TestFiles::new();
        mocks.file(
            "index.yml",
            indoc! {"
                ---
                1
            "},
        );
        let v: i32 = mocks.resolver().get(&[])?;
        assert_eq!(v, 1);
        Ok(())
    }

    #[test]
    fn resolves_list_num_accross_files() -> Result<()> {
        let mocks = TestFiles::new();
        // See above comment about in future chosing not this behaviour
        mocks
            .file(
                "a.yml",
                indoc! {"
	            ---
	            1
	        "},
            )
            .file(
                "b.yml",
                indoc! {"
	            ---
	            2
	        "},
            );

        let mut v: Vec<i32> = mocks.resolver().get(&[])?;
        // we get not guarantee on order with file iterator
        v.sort();
        assert_eq!(v, vec![1, 2]);
        Ok(())
    }

    #[test]
    fn resolves_object_from_index() -> Result<()> {
        let mocks = TestFiles::new();
        mocks.file(
            "index.yml",
            indoc! {"
                ---
                id: 1
                name: Objy
            "},
        );
        let v: MyObj = mocks.resolver().get(&[])?;
        assert_eq!(
            v,
            MyObj {
                id: 1,
                name: "Objy".to_owned()
            }
        );
        Ok(())
    }

    #[test]
    fn resolves_object_from_broken_files() -> Result<()> {
        let mocks = TestFiles::new();
        mocks
            .file(
                "id.yml",
                indoc! {"
                ---
                1
            "},
            )
            .file(
                "name.yml",
                indoc! {"
                ---
                Objy
            "},
            );
        let v: MyObj = mocks.resolver().get(&[])?;
        assert_eq!(
            v,
            MyObj {
                id: 1,
                name: "Objy".to_owned()
            }
        );
        Ok(())
    }

    #[test]
    fn resolves_deep_object_from_index() -> Result<()> {
        let mocks = TestFiles::new();
        mocks.file(
            "index.yml",
            indoc! {"
                ---
                my_obj:
                    id: 1
                    name: Objy
                my_list:
                - id: 1
                  alias: Obbo
                - id: 2
                  alias: Ali
            "},
        );
        let v: Query = mocks.resolver().get(&[])?;
        assert_eq!(
            v,
            Query {
                my_obj: MyObj {
                    id: 1,
                    name: "Objy".to_owned()
                },
                my_list: vec![
                    MyOtherObj {
                        id: 1,
                        alias: "Obbo".to_owned(),
                    },
                    MyOtherObj {
                        id: 2,
                        alias: "Ali".to_owned(),
                    },
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn resolves_nested_list_from_files() -> Result<()> {
        let mocks = TestFiles::new();
        mocks
            .file(
                "my_obj/index.yml",
                indoc! {"
                ---
                id: 1
                name: Objy
            "},
            )
            .file(
                "my_list/x.yml",
                indoc! {"
                ---
                id: 1
                alias: Obbo
            "},
            )
            .file(
                "my_list/y.yml",
                indoc! {"
                ---
                id: 2
                alias: Ali
            "},
            );
        let mut v: Query = mocks.resolver().get(&[])?;
        v.my_list.sort();
        assert_eq!(
            v,
            Query {
                my_obj: MyObj {
                    id: 1,
                    name: "Objy".to_owned()
                },
                my_list: vec![
                    MyOtherObj {
                        id: 1,
                        alias: "Obbo".to_owned(),
                    },
                    MyOtherObj {
                        id: 2,
                        alias: "Ali".to_owned(),
                    },
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn resolves_broken_nested_list_from_dir_index_files() -> Result<()> {
        let mocks = TestFiles::new();
        mocks
            .file(
                "my_obj/index.yml",
                indoc! {"
                ---
                id: 1
                name: Objy
            "},
            )
            .file(
                "my_list/x/index.yml",
                indoc! {"
                ---
                id: 1
                alias: Obbo
            "},
            )
            .file(
                "my_list/y/index.yml",
                indoc! {"
                ---
                id: 2
                alias: Ali
            "},
            );
        let mut v: Query = mocks.resolver().get(&[])?;
        v.my_list.sort();
        assert_eq!(
            v,
            Query {
                my_obj: MyObj {
                    id: 1,
                    name: "Objy".to_owned()
                },
                my_list: vec![
                    MyOtherObj {
                        id: 1,
                        alias: "Obbo".to_owned(),
                    },
                    MyOtherObj {
                        id: 2,
                        alias: "Ali".to_owned(),
                    },
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn resolves_broken_nested_list_from_dir_tree() -> Result<()> {
        let mocks = TestFiles::new();
        mocks
            .file(
                "my_obj/index.yml",
                indoc! {"
                ---
                id: 1
                name: Objy
            "},
            )
            .file(
                "my_list/x/index.yml",
                indoc! {"
                ---
                id: 1
            "},
            )
            .file(
                "my_list/x/alias.yml",
                indoc! {"
                ---
                Obbo
            "},
            )
            .file(
                "my_list/y/alias.yml",
                indoc! {"
                ---
                Ali
            "},
            )
            .file(
                "my_list/y/id.yml",
                indoc! {"
                ---
                2
            "},
            );
        let mut v: Query = mocks.resolver().get(&[])?;
        v.my_list.sort();
        assert_eq!(
            v,
            Query {
                my_obj: MyObj {
                    id: 1,
                    name: "Objy".to_owned()
                },
                my_list: vec![
                    MyOtherObj {
                        id: 1,
                        alias: "Obbo".to_owned(),
                    },
                    MyOtherObj {
                        id: 2,
                        alias: "Ali".to_owned(),
                    },
                ]
            }
        );
        Ok(())
    }
}
