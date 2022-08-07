use itertools::Itertools;
pub use serde_yaml::Value;
use std::path::Path;

use super::DataResolverError;

pub trait ValueFilter: Fn(&Value) -> bool {}

impl<T> ValueFilter for T where T: Fn(&Value) -> bool {}

// TODO:
// struct FilteredValue {
//    value: Value,
//    filters: Vec<ValueFilter>
// }
//
// and write a short-circuit version of serde_yaml::from_reader ?

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

pub fn value_from_file(path: &Path) -> Result<Value, DataResolverError> {
    let file = std::fs::File::open(&path)?;
    let value = serde_yaml::from_reader::<_, Value>(file)?;
    Ok(value)
}

/// Define methods for
/// * merging in values from an instance of an associated type (may be `Self`)
/// * doing the above but instead under a specified key within the target instance
/// * taking value from a mutable ref
pub trait Merge {
    /// The type from which we'll merge values into `self`
    type Other;
    /// Mutate `self` by merging in `mergee`
    fn merge(&mut self, mergee: Self::Other) -> Result<&mut Self, DataResolverError>;
    /// Mutate `self` by merging in `mergee` to a specified key
    fn merge_at(&mut self, key: &str, mergee: Self::Other) -> Result<&mut Self, DataResolverError>;
    /// Take ownership via mutable reference
    fn take(&mut self) -> Self;
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
    type Other = Self;

    fn merge(&mut self, mut mergee: Self::Other) -> Result<&mut Self, DataResolverError> {
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
    fn merge_at(&mut self, key: &str, mergee: Self::Other) -> Result<&mut Self, DataResolverError> {
        match self {
            Self::Mapping(mapping) => {
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
            }
            Self::Null => {
                let mut mapping = serde_yaml::Mapping::new();
                mapping.insert(key.into(), mergee);
                *self = Self::Mapping(mapping);
                Ok(self)
            }
            _ => Err(DataResolverError::CannotMergeIntoNonMapping(self.clone())),
        }
    }
    /// Returns owned [serde_yaml::Value], leaving [serde_yaml::Value::Null] in
    /// its place.
    fn take(&mut self) -> Self {
        std::mem::replace(self, serde_yaml::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
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
        let mocks = TestFiles::new();
        mocks.file(filename, content);
        let file_path = mocks.path().join(filename);

        let read_value = value_from_file(&file_path)?;

        assert_eq!(serde_yaml::to_string(&read_value)?, content);
        Ok(())
    }

    #[test]
    fn takes_sub_value_at_address() -> Result<()> {
        let mut value = yaml! {"
            ---
            my:
                yaml:
                    is:
                        - hella
                        - deep
        "};

        let taken = take_sub_value_at_address(&mut value, &["my", "yaml"])?;

        assert_eq!(
            taken,
            yaml! {"
            ---
            is:
                - hella
                - deep
        "}
        );

        assert_eq!(
            value,
            yaml! {"
            ---
            my:
                yaml:
        "}
        );

        Ok(())
    }
}
