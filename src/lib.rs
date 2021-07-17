use async_graphql::{Object, SimpleObject};

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

    async fn heroes(&self) -> Vec<Hero> {
        vec![Hero {
            name: "Bobby".to_owned(),
            powers: vec![],
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{value, EmptyMutation, EmptySubscription, Schema};

    #[actix_rt::test]
    async fn hello() {
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let res = schema.execute("{ add(a: 10, b: 20) }").await;
        assert_eq!(res.data, value!({"add": 30}));
    }

    #[actix_rt::test]
    async fn test_test() {
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let res = schema.execute("{ heroes { name } }").await;
        assert_eq!(res.data, value!({"heroes": [{"name": "Bobby"}]}));
    }
}
