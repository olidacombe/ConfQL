use color_eyre::Result;
use confql::graphql_schema;
use indoc::indoc;
use juniper::{graphql_value, EmptyMutation, EmptySubscription};
use test_files::TestFiles;

graphql_schema! {
    type Thing {
        name: String @confql(arrayFilename: true)
        size: Float!
    }

    type Query {
        things: [Thing!]!
    }

    schema {
        query: Query
    }
}

fn main() -> Result<()> {
    let mocks = TestFiles::new();
    mocks
        .file(
            "things/widget.yml",
            indoc! {"
                ---
                size: 1.1
            "},
        )
        .file(
            "things/dongle.yml",
            indoc! {"
                ---
                size: 2.2
            "},
        );

    let ctx = Ctx::from(mocks.path().to_path_buf());

    // Run the executor.
    let (res, _errors) = juniper::execute_sync(
        indoc! {"
            {
                things {
                    name
                    size
                }
            }"},
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
            "things": [
                {"name": "widget", "size": 1.1},
                {"name": "dongle", "size": 2.2}
            ]
        })
    );

    Ok(())
}
