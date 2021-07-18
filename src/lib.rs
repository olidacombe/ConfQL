use async_graphql::{Object, SimpleObject};
#[macro_use]
extern crate lazy_static;

struct Settings<'a> {
    index_filenames: Option<Vec<&'a str>>,
}

const DEFAULT_INDEX_FILENAMES: &'static [&str] = &["index.yml"];

impl Settings<'_> {
    fn new() -> Self {
        Self {
            index_filenames: Some(DEFAULT_INDEX_FILENAMES.to_vec()),
        }
    }
}

lazy_static! {
    static ref SETTINGS: Settings<'static> = Settings::new();
}

#[derive(SimpleObject)]
struct Hero {
    name: String,
    powers: Vec<String>,
}

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
            powers: vec![],
        }]
    }

    async fn heroes(&self) -> Vec<Hero> {
        vec![]
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
