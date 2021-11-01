use color_eyre::Result;
use confql_proc_macro::graphql_schema;
use indoc::indoc;
use juniper::{graphql_value, EmptyMutation, EmptySubscription, Value};
use test_files::TestFiles;

graphql_schema! {
    type Thing {
        name: String @confql(arrayFilename: true)
        alias: String @confql(arrayFilename: true)
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
                alias: Widgy
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
    let (mut res, _errors) = juniper::execute_sync(
        indoc! {"
            {
                things {
                    name
                    alias
                    size
                }
            }"},
        None,
        &Schema::new(Query, EmptyMutation::new(), EmptySubscription::new()),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    let things = res
        .as_mut_object_value()
        .unwrap()
        .get_mut_field_value("things")
        .unwrap();
    let mut new_things = things.as_list_value().unwrap().clone();
    new_things.sort_by(|a, b| {
        a.as_object_value()
            .unwrap()
            .get_field_value("name")
            .unwrap()
            .as_string_value()
            .unwrap()
            .cmp(
                b.as_object_value()
                    .unwrap()
                    .get_field_value("name")
                    .unwrap()
                    .as_string_value()
                    .unwrap(),
            )
    });
    *things = Value::list(new_things);

    // Ensure the value matches.
    assert_eq!(
        res,
        graphql_value!({
            "things": [
                {"name": "dongle", "alias": "dongle", "size": 2.2},
                {"name": "widget", "alias": "Widgy", "size": 1.1}
            ]
        })
    );

    Ok(())
}
