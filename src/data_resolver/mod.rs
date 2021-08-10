mod data_path;

use anyhow::{Context, Error, Result};
use data_path::DataPath;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::Path;

macro_rules! typename {
    ($T:ty) => {
        std::any::type_name::<$T>()
    };
}

/// Returns reference to sub-value of a deserialized Value
fn get_sub_value<'a>(value: &'a Value, index: &[&str]) -> Result<&'a Value> {
    return index
        .iter()
        .fold_while(Ok(value), |acc, i| match acc.unwrap().get(i) {
            Some(v) => Continue(Ok(v)),
            _ => Done(Err(Error::msg(format!("Key {} not found", i,)))),
        })
        .into_inner();
}

fn get_object_from_path<T>(path: &Path, index: &[&str]) -> Result<T>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    let file = std::fs::File::open(path)?;
    let value = serde_yaml::from_reader::<_, Value>(file)?;
    let object = get_sub_value(&value, index)?;
    let object: T = serde_yaml::from_value(object.to_owned())
        .context(format!("Failed to deserialize to {}", typename!(T)))?;
    Ok(object)
}

#[cfg(test)]
mod tests {
    extern crate fixtures;

    use super::*;
    use fixtures::models::Hero;
    use fixtures::DATA_PATH;
    use std::fs;

    #[test]
    fn test_get_sub_value() -> Result<()> {
        use crate::data_resolver::get_sub_value;
        use crate::yaml;
        let value = yaml!(
            r"#---
            A:
                B:
                    C:
                        presence: welcome
            #"
        );

        let sub_value = get_sub_value(&value, &vec![])?;
        assert_eq!(*sub_value, value);

        let sub_value = get_sub_value(&value, &vec!["A", "B"])?;
        assert_eq!(
            *sub_value,
            yaml!(
                r#"---
                C:
                    presence: welcome
                "#
            )
        );
        Ok::<(), anyhow::Error>(())
    }

    #[test]
    fn test_get_object_from_path() -> Result<()> {
        let result: Vec<Hero> = get_object_from_path(&DATA_PATH.join("index.yml"), &["heroes"])?;

        assert_eq!(
            result,
            vec![Hero {
                // index.yml
                name: "Andy Anderson".to_owned(),
                id: 1,
                powers: vec!["eating".to_owned(), "sleeping".to_owned()]
            }]
        );
        // assert_eq!(
        //     results,
        //     vec![
        //         Some(vec![Hero {
        //             // index.yml
        //             name: "Andy Anderson".to_owned(),
        //             id: 1,
        //             powers: vec!["eating".to_owned(), "sleeping".to_owned()]
        //         }]),
        //         None, // heroes.yml (doesn't exist)
        //         Some(vec![Hero {
        //             // heroes/charles.yml
        //             name: "Charles Charleston".to_owned(),
        //             id: 3,
        //             powers: vec!["moaning".to_owned(), "cheating".to_owned()]
        //         }]),
        //         Some(vec![Hero {
        //             // heroes/kevin.yml
        //             name: "Kevin Kevinson".to_owned(),
        //             id: 2,
        //             powers: vec!["hunting".to_owned(), "fighting".to_owned()]
        //         }]),
        //     ]
        // );

        Ok(())
    }
}
