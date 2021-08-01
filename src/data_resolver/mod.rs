mod data_path;

use anyhow::{Error, Result};
use data_path::DataPath;

struct DataResolver<'a> {
    data_path: DataPath<'a>,
}

#[cfg(test)]
mod tests {
    extern crate fixtures;

    use super::*;
    use fixtures::models::Hero;
    use fixtures::DATA_PATH;
    use std::fs;
    use temp_testdir::TempDir;

    #[test]
    fn data_resolver_iterator() -> Result<()> {
        type T = Vec<Hero>;
        let resolver = DataPath::new(&DATA_PATH, vec!["heroes"]);
        let mut results = Vec::<Option<T>>::new();

        //for result in resolver {
        //results.push(result);
        //}

        assert_eq!(
            results,
            vec![
                Some(vec![Hero {
                    // index.yml
                    name: "Andy Anderson".to_owned(),
                    id: 1,
                    powers: vec!["eating".to_owned(), "sleeping".to_owned()]
                }]),
                None, // heroes.yml (doesn't exist)
                Some(vec![Hero {
                    // heroes/charles.yml
                    name: "Charles Charleston".to_owned(),
                    id: 3,
                    powers: vec!["moaning".to_owned(), "cheating".to_owned()]
                }]),
                Some(vec![Hero {
                    // heroes/kevin.yml
                    name: "Kevin Kevinson".to_owned(),
                    id: 2,
                    powers: vec!["hunting".to_owned(), "fighting".to_owned()]
                }]),
            ]
        );

        Ok(())
    }
}
