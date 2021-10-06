use itertools::Itertools;
use serde_yaml::Value;
use std::path::PathBuf;

use super::DataResolverError;

pub fn get_sub_value_at_address<'a>(
    value: &'a Value,
    address: &[&str],
) -> Result<&'a Value, DataResolverError> {
    use itertools::FoldWhile::{Continue, Done};
    return address
        .iter()
        .fold_while(Ok(value), |acc, i| match acc.unwrap().get(i) {
            Some(v) => Continue(Ok(v)),
            _ => Done(Err(DataResolverError::KeyNotFound(i.to_string()))),
        })
        .into_inner();
}

pub fn take_sub_value_at_address(
    value: &mut Value,
    address: &[&str],
) -> Result<Value, DataResolverError> {
    use itertools::FoldWhile::{Continue, Done};
    return address
        .iter()
        .fold_while(Ok(value), |acc, i| match acc.unwrap().get_mut(i) {
            Some(v) => Continue(Ok(v)),
            _ => Done(Err(DataResolverError::KeyNotFound(i.to_string()))),
        })
        .into_inner()
        .map(|v| std::mem::replace(v, Value::Null));
}

pub fn value_from_file(path: &PathBuf) -> Result<Value, DataResolverError> {
    let file = std::fs::File::open(&path)?;
    let value = serde_yaml::from_reader::<_, Value>(file)?;
    Ok(value)
}

pub trait Merge {
    fn merge(&mut self, mergee: Self) -> Result<&mut Self, DataResolverError>;
    fn merge_at(&mut self, key: &str, mergee: Self) -> Result<&mut Self, DataResolverError>;
}

macro_rules! merge_compat_err {
    ($self:expr, $mergee:expr) => {
        Err(DataResolverError::IncompatibleYamlMerge {
            dst: $self.clone(),
            src: $mergee,
        })
    };
}

impl Merge for serde_yaml::Value {
    fn merge(&mut self, mut mergee: Self) -> Result<&mut Self, DataResolverError> {
        use serde_yaml::Value::{Bool, Mapping, Null, Number, Sequence, String};
        if let Null = mergee {
            return Ok(self);
        }
        match self {
            Null => {
                *self = mergee;
            }
            Bool(_) => {
                if !mergee.is_bool() {
                    return merge_compat_err! {self, mergee};
                }
                *self = mergee;
            }
            Number(_) => {
                if !mergee.is_number() {
                    return merge_compat_err! {self, mergee};
                }
                *self = mergee;
            }
            String(_) => {
                if !mergee.is_string() {
                    return merge_compat_err! {self, mergee};
                }
                *self = mergee;
            }
            Sequence(list) => {
                if let Sequence(ref mut appendee) = mergee {
                    list.append(appendee);
                } else {
                    return merge_compat_err! {self, mergee};
                }
            }
            Mapping(mapping) => {
                if let Mapping(superimposee) = mergee {
                    for (key, src) in superimposee {
                        if let Some(dst) = mapping.get_mut(&key) {
                            dst.merge(src)?;
                        } else {
                            mapping.insert(key, src);
                        }
                    }
                } else {
                    return merge_compat_err! {self, mergee};
                }
            }
        };
        Ok(self)
    }
    fn merge_at(&mut self, key: &str, mergee: Self) -> Result<&mut Self, DataResolverError> {
        if let Self::Mapping(mapping) = self {
            let key: Self = key.into();
            match mapping.get_mut(&key) {
                Some(value) => {
                    value.merge(mergee)?;
                }
                None => {
                    mapping.insert(key, mergee);
                }
            }
            Ok(self)
        } else {
            Err(DataResolverError::CannotMergeIntoNonMapping(self.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use indoc::indoc;
    use test_files::TestFiles;
    use test_utils::yaml;

    // TODO beyond just a couple of happy tests

    #[test]
    fn gets_value_from_file() -> Result<()> {
        let filename = "index.yml";
        let content = indoc! {"
			---
			ok: true
			go: home
		"};
        let mocks = TestFiles::new().unwrap();
        mocks.file(filename, content)?;
        let file_path = mocks.path().join(filename);

        let read_value = value_from_file(&file_path)?;

        assert_eq!(serde_yaml::to_string(&read_value)?, content);
        Ok(())
    }

    #[test]
    fn gets_sub_value_at_address() -> Result<()> {
        let value = yaml! {"
            ---
            my:
                yaml:
                    is:
                    - hella
                    - deep
		"};

        assert_eq!(
            get_sub_value_at_address(&value, &["my", "yaml", "is"])?,
            &yaml! {"
        		---
        		- hella
        		- deep
        	"}
        );
        Ok(())
    }
}
