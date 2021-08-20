mod data_path;

use anyhow::{Context, Error, Result};
use data_path::DataPath;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::Path;

// impl DataPath<'_> {
//     fn get_object<T>(&self) -> Result<T>
//     where
//         T: for<'de> Deserialize<'de> + std::fmt::Debug,
//     {
//         let file = self.open()?;
//         let value = serde_yaml::from_reader::<_, Value>(file)
//             .with_context(|| format!("Failed to parse {}", &self))?;
//         let object = get_sub_value_reverse_index(&value, &self.key_path())?;
//         let object: T = serde_yaml::from_value(object.to_owned())
//             .context(format!("Failed to deserialize to {}", typename!(T)))?;
//         Ok(object)
//     }
// }

#[cfg(test)]
mod tests {
    extern crate fixtures;

    use super::*;
    use fixtures::models::Hero;
    use fixtures::DATA_PATH;
    use std::fs;

    #[test]
    fn test_data_path_get_object() -> Result<()> {
        // let result: Vec<Hero> = DataPath::new(&DATA_PATH, vec!["heroes"])?.get_object()?;

        // assert_eq!(
        //     result,
        //     vec![Hero {
        //         name: "Andy Anderson".to_owned(),
        //         id: 1,
        //         powers: vec!["eating".to_owned(), "sleeping".to_owned()]
        //     }]
        // );

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
