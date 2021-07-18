#[macro_use]
extern crate lazy_static;

use async_graphql::{Object, SimpleObject};
use colored::Colorize;
use serde_yaml;
use std::path::{Path, PathBuf};

struct Settings<'a> {
    index_filenames: Vec<&'a str>,
    root: &'a Path,
}

const DEFAULT_INDEX_FILENAMES: &'static [&str] = &["index.yml", "shitos"];

impl Settings<'_> {
    fn new() -> Self {
        Self {
            index_filenames: DEFAULT_INDEX_FILENAMES.to_vec(),
            root: &DEFAULT_ROOT,
        }
    }
}

lazy_static! {
    static ref SETTINGS: Settings<'static> = Settings::new();
    static ref DEFAULT_ROOT: &'static Path = Path::new("./data");
}

#[derive(SimpleObject)]
struct Hero {
    name: String,
    powers: Vec<String>,
}

struct Query;

async fn get_heroes_from_path(path: PathBuf) -> Option<Vec<Hero>> {
    if let Ok(f) = std::fs::File::open(path) {
        if let Ok(d) = serde_yaml::from_reader::<_, serde_yaml::Value>(f) {
            eprintln!("{}", "OKAAAAY".magenta());
        }
    } else {
        eprintln!("{}", "FUUUUCK".purple());
    }
    //}
    //
    //println!("Read YAML string: {}", d);
    //}
    Some(vec![])
}

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    async fn losers(&self) -> Vec<Hero> {
        vec![Hero {
            name: "Bobby".to_owned(),
            powers: vec![],
        }]
    }

    async fn heroes(&self) -> Vec<Hero> {
        let mut heroes: Vec<Hero> = vec![];
        for index_filename in SETTINGS.index_filenames.iter() {
            if let Some(hs) = get_heroes_from_path(SETTINGS.root.join(index_filename)).await {
                heroes.extend(hs);
            }
        }
        heroes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{value, EmptyMutation, EmptySubscription, Schema};

    macro_rules! assert_query_result {
        ($a:expr, $b:expr) => {
            let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
            let res = schema.execute($a).await;
            assert_eq!(res.data, $b);
        };
    }

    #[actix_rt::test]
    async fn hello() {
        assert_query_result!("{ add(a: 10, b: 20) }", value!({"add": 30}));
    }

    #[actix_rt::test]
    async fn test_test_type_comparison() {
        assert_query_result!(
            "{ losers { name } }",
            value!({"losers": [{"name": "Bobby"}]})
        );
    }

    #[actix_rt::test]
    async fn finds_heroes() {
        assert_query_result!(
            "{ heroes { name } }",
            value!({"heroes": [
                { "name": "Andy Anderson" },
                { "name": "Charlie Charleston" },
                { "name": "Kevin Kevinson" },
            ]})
        );
    }
}
