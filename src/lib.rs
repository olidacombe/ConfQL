#[macro_use]
extern crate lazy_static;

use anyhow::{Context, Error, Result};
use async_graphql::{Object, SimpleObject};
use colored::Colorize;
use serde::Deserialize;
use serde_yaml::Value;
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

// E.g.
// path: _, index: A.B.C
// + index.yml :: A.B.C
// + {A.yml, A/index.yml} :: B.C
// + {A/B.yml, A/B/index.yml} :: C
//     + {A/B/C.yml, A/B/C/index.yml} :: _ # C not array
//     + {A/B/C.yml, A/B/C/*.yml (each file as entry)} :: _ # C array

// TODO
// for this fancy id shit, maybe we encode the serde_yaml => hydration bits
// into a trait method.  Then if our target type implements IdFromFilename (wevs)
// then work it so the id retrofitting gets called before the serde_yaml::from_value
// somehow.

//fn get_sub_value(value &Value, index: &vec![&str]) -> Result<&Value> {
// TODO fuck it, just recurse on slices

//}

// TODO
// - eat index, building path, and merging up
// - indicator of whether we're looking for an array
//     - if so then when index is empty (e.g. heroes directory)
//       assume each file is an array item
// - indicator of what field a filename should provide a default for
//   in the case of above array scenario
async fn get_object_from_path<T>(path: &PathBuf, index: vec![&str]) -> Result<T>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    let f = std::fs::File::open(path)?;
    let d = serde_yaml::from_reader::<_, Value>(f)?;
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
            match get_object_from_path::<Vec<Hero>>(
                &SETTINGS.root.join(index_filename),
                vec!["heroes"],
            )
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
