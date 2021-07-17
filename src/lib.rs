use async_graphql::Object;

struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
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
}
