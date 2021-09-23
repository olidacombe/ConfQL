use confql::graphql_schema;

graphql_schema! {
    type Query {
    name: String!
    }

    schema {
        query: Query
    }
}

fn main() {
    let _ = Query {
        name: "douggy".to_owned(),
    };
}
