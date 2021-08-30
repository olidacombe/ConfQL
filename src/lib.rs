#![feature(specialization)]
#[macro_use]
extern crate lazy_static;

mod data_resolver;

//struct Settings<'a> {
//index_filenames: Vec<&'a str>,
//root: &'a Path,
//}

//const DEFAULT_INDEX_FILENAMES: &[&str] = &["index.yml", "shitos"];

//impl Settings<'_> {
//fn new() -> Self {
//Self {
//index_filenames: DEFAULT_INDEX_FILENAMES.to_vec(),
//root: &DEFAULT_ROOT,
//}
//}
//}

//lazy_static! {
//static ref SETTINGS: Settings<'static> = Settings::new();
//static ref DEFAULT_ROOT: &'static Path = Path::new("./data");
//}

// E.g.  path: _, index: A.B.C
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

// TODO only expose in tests / doctests?
#[macro_export]
macro_rules! yaml {
    ($e:literal) => {
        serde_yaml::from_str::<serde_yaml::Value>($e).unwrap()
    };
}

#[cfg(test)]
mod tests {
    extern crate fixtures;

    use super::*;
    use async_graphql::{value, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
    use fixtures::models::Hero;

    struct Query;

    #[Object]
    impl Query {
        /// Returns the sum of a and b
        async fn add(&self, a: i32, b: i32) -> i32 {
            a + b
        }

        async fn losers(&self) -> Vec<Hero> {
            vec![Hero {
                name: "Bobby".to_owned(),
                id: 99,
                powers: vec![],
            }]
        }

        async fn heroes(&self) -> Vec<Hero> {
            let mut heroes: Vec<Hero> = vec![];
            //for index_filename in SETTINGS.index_filenames.iter() {
            //match get_object_from_path::<Vec<Hero>>(
            //&SETTINGS.root.join(index_filename),
            //&["heroes"],
            //)
            //.await
            //{
            //Ok(hs) => heroes.extend(hs),
            //Err(e) => eprintln!("{}", e),
            //}
            //}
            heroes
        }
    }

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

    // TODO
    //#[actix_rt::test]
    //async fn finds_heroes() {
    //assert_query_result!(
    //"{ heroes { name } }",
    //value!({"heroes": [
    //{ "name": "Andy Anderson" },
    //{ "name": "Charlie Charleston" },
    //{ "name": "Kevin Kevinson" },
    //]})
    //);
    //}
}
