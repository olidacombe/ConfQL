use confql::graphql_schema;

graphql_schema! {
    type Thing {
        name: String!
    }

    type Query {
        thing: Thing!
    }

    schema {
        query: Query
    }
}

fn main() {
    let _ = Thing {
        name: "douggy".to_owned(),
    };
}
