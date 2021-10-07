use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

mod data_path;
use data_path::DataPath;
mod values;
use values::Merge;

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

pub struct DataResolver<'a> {
    root: &'a Path,
}

impl<'a> DataResolver<'a> {
    // pub fn get_non_nullable<T>(&self, address: &[&str]) -> Result<T, DataResolverError>
    // where
    //     T: for<'de> Deserialize<'de>,
    // {
    //     self.get_nullable(address)?
    //         .ok_or(DataResolverError::DataNotFound)
    // }

    // pub fn get_non_nullable_list<T>(&self, address: &[&str]) -> Result<Vec<T>, DataResolverError>
    // where
    //     T: for<'de> Deserialize<'de>,
    // {
    //     self.get_nullable_list(address)?
    //         .ok_or(DataResolverError::DataNotFound)
    // }

    pub fn get<T>(&self, address: &[&str]) -> Result<T, DataResolverError>
    where
        T: for<'de> Deserialize<'de>,
        T: ResolveValue,
    {
        let data_path = DataPath::new(self.root, address);
        let value = T::resolve_value(data_path)?;
        Ok(serde_yaml::from_value(value)?)
    }

    // pub fn get_nullable_list<T>(
    //     &self,
    //     address: &[&str],
    // ) -> Result<Option<Vec<T>>, DataResolverError>
    // where
    //     T: for<'de> Deserialize<'de>,
    // {
    //     Err(DataResolverError::DataNotFound)
    // }
}

impl<'a> From<&'a Path> for DataResolver<'a> {
    fn from(path: &'a Path) -> Self {
        Self { root: path }
    }
}

pub trait ResolveValue {
    fn merge_properties(
        _value: &mut serde_yaml::Value,
        _data_path: &DataPath,
    ) -> Result<(), DataResolverError> {
        Ok(())
    }
    fn resolve_value(mut data_path: DataPath) -> Result<serde_yaml::Value, DataResolverError> {
        let mut value = serde_yaml::Value::Null;
        if data_path.done() {
            Self::merge_properties(&mut value, &data_path)?;
        } else {
            value = data_path.value();
            data_path.next();
            value.merge(Self::resolve_value(data_path)?)?;
        }
        Ok(value)
    }
    fn resolve_values(mut data_path: DataPath) -> Result<serde_yaml::Value, DataResolverError> {
        let mut value = data_path.values();
        if !data_path.done() {
            data_path.next();
            value.merge(Self::resolve_values(data_path)?)?;
        }
        Ok(value)
    }
}

impl ResolveValue for bool {}
impl ResolveValue for f64 {}
// TODO?
// impl ResolveValue for juniper::ID {}
impl ResolveValue for String {}
impl ResolveValue for u32 {}

#[cfg(test)]
mod tests {
    use super::values::Merge;
    use super::*;
    use anyhow::Result;
    use indoc::indoc;
    use test_files::TestFiles;

    // TODO macro generates the below automatically for
    // such types

    #[derive(Deserialize, Debug)]
    struct MyObj {
        id: u32,
        name: String,
    }

    impl ResolveValue for MyObj {
        fn merge_properties(
            value: &mut serde_yaml::Value,
            data_path: &DataPath,
        ) -> Result<(), DataResolverError> {
            value.merge_at("id", u32::resolve_value(data_path.join("id"))?)?;
            value.merge_at("name", String::resolve_values(data_path.join("name"))?)?;
            Ok(())
        }
    }

    #[derive(Deserialize, Debug)]
    struct MyOtherObj {
        id: u32,
        alias: String,
    }

    impl ResolveValue for MyOtherObj {
        fn merge_properties(
            value: &mut serde_yaml::Value,
            data_path: &DataPath,
        ) -> Result<(), DataResolverError> {
            value.merge_at("id", u32::resolve_value(data_path.join("id"))?)?;
            value.merge_at("alias", String::resolve_values(data_path.join("alias"))?)?;
            Ok(())
        }
    }

    #[derive(Deserialize, Debug)]
    struct Query {
        my_obj: MyObj,
        my_list: Vec<MyOtherObj>,
    }

    impl ResolveValue for Query {
        fn merge_properties(
            value: &mut serde_yaml::Value,
            data_path: &DataPath,
        ) -> Result<(), DataResolverError> {
            value.merge_at("my_obj", MyObj::resolve_value(data_path.join("my_obj"))?)?;
            value.merge_at(
                "my_list",
                MyOtherObj::resolve_values(data_path.join("my_list"))?,
            )?;
            Ok(())
        }
    }

    trait GetResolver<'a> {
        fn resolver(&'a self) -> DataResolver<'a>;
    }

    impl<'a> GetResolver<'a> for TestFiles {
        fn resolver(&'a self) -> DataResolver<'a> {
            DataResolver { root: self.path() }
        }
    }

    #[test]
    fn resolves_object_from_index() -> Result<()> {
        let mocks = TestFiles::new().unwrap();
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
        )?;
        let v: Query = mocks.resolver().get(&[])?;
        Ok(())
    }
}
