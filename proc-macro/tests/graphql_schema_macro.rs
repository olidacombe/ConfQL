use confql::graphql_schema;

graphql_schema! {
    type Query {
        version: String!
    }

    schema {
        query: Query
    }
}

fn main() {}
