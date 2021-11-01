use confql_proc_macro::graphql_schema;

graphql_schema! {
    type Query {
        version: String!
    }

    schema {
        query: Query
    }
}

fn main() {}
