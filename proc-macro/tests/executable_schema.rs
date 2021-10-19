use confql::graphql_schema;
use indoc::indoc;
use juniper::{graphql_value, EmptyMutation, EmptySubscription};
use test_files::TestFiles;

graphql_schema! {
    type Query {
        id: String!
    }

    schema {
        query: Query
    }
}

fn main() {
    let mocks = TestFiles::new();
    mocks.file(
        "id.yml",
        indoc! {"
                ---
                MegaString
            "},
    );

    let ctx = Ctx::from(mocks.path().to_path_buf());

    // Run the executor.
    let (res, _errors) = juniper::execute_sync(
        "query { id }",
        None,
        &Schema::new(Query, EmptyMutation::new(), EmptySubscription::new()),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    // Ensure the value matches.
    assert_eq!(
        res,
        graphql_value!({
            "id": "MegaString"
        })
    );
}
