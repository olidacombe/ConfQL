use color_eyre::Result;
use confql::graphql_schema;
use indoc::indoc;
use juniper::{graphql_value, EmptyMutation, EmptySubscription};
use test_files::TestFiles;

graphql_schema! {
    type Thing {
        name: String
        size: Float!
    }

    type Compartment {
        name: String
        things: [Thing]
    }

    type Query {
        compartments: [Compartment]
    }

    schema {
        query: Query
    }
}

fn main() -> Result<()> {
    let mocks = TestFiles::new().unwrap();
    mocks
        .file(
            "compartments/A/index.yml",
            indoc! {"
                ---
                name: A
            "},
        )?
        .file(
            "compartments/A/things/1.yml",
            indoc! {"
                ---
                name: One
                size: 1.1
            "},
        )?
        .file(
            "compartments/B/index.yml",
            indoc! {"
                ---
                name: B
            "},
        )?;

    let ctx = Ctx::from(mocks.path().to_path_buf());

    // Run the executor.
    let (res, _errors) = juniper::execute_sync(
        indoc! {"
            {
                compartments {
                    name
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
            "compartments": [
                {"name": "A"},
                {"name": "B"}
            ]
        })
    );

    // Run the executor.
    let (res, _errors) = juniper::execute_sync(
        indoc! {"
            {
                compartments {
                    name
                    things {
                        name
                        size
                    }
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
            "compartments": [
                {"name": "A", "things": [
                    {"name": "One", "size": 1.1},
                ]},
                {"name": "B", "things": None}
            ]
        })
    );

    Ok(())
}
