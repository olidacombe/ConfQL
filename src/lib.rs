#[macro_use]
extern crate lazy_static;

use anyhow::{Context, Error, Result};
use async_graphql::{Object, SimpleObject};
use colored::Colorize;
use serde::Deserialize;
use std::path::{Path, PathBuf};

macro_rules! typename {
    ($T:ty) => {
        std::any::type_name::<$T>()
    };
}

struct Settings<'a> {
    index_filenames: Vec<&'a str>,
    root: &'a Path,
}

const DEFAULT_INDEX_FILENAMES: &[&str] = &["index.yml", "shitos"];

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

#[derive(SimpleObject, Deserialize, Debug)]
struct Hero {
    name: String,
    powers: Vec<String>,
}

struct Query;

async fn get_object_from_path<T>(path: &PathBuf, index: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    let f = std::fs::File::open(path)?;
    let d = serde_yaml::from_reader::<_, serde_yaml::Value>(f)?;
    if let Some(object) = d.get(index) {
        //eprintln!(
        //"{}\n{}",
        //"shit YEAAAAH".yellow(),
        //serde_yaml::to_string(object)?.purple()
        //);
        let object: T = serde_yaml::from_value(object.to_owned())
            .context(format!("Failed to deserialize to {}", typename!(T)))?;
        //eprintln!("{}, {:?}", "coool YEAAAAH".blue(), object);
        Ok(object)
    } else {
        Err(Error::msg(format!(
            "No {} found at {}::{}",
            typename!(T),
            path.display(),
            index,
        )))
    }
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
            match get_object_from_path::<Vec<Hero>>(&SETTINGS.root.join(index_filename), "heroes")
                .await
            {
                Ok(hs) => heroes.extend(hs),
                Err(e) => eprintln!("{}", e),
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
