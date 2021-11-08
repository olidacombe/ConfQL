## ConfQL

This is intended as a very low-friction means of turning structured yaml into a [GraphQL](https://graphql.org/) service.

> Your first place to look should be the quick start [example](https://github.com/olidacombe/ConfQL/tree/main/example)
> in the source repo.  There you'll find a way to containerize a GraphQL service
> given only a quick container build against your schema, run with a data directory mount.

The client surface to this library is pretty much just the procedural macro
[graphql_schema_from_file].

## Example

```rust
use confql::graphql_schema;
use indoc::indoc;
use juniper::{graphql_value, EmptyMutation, EmptySubscription};
use test_files::TestFiles;

// In practice you'll more likely use the `graphql_schema_from_file` macro
// but this macro is nice for tests.
graphql_schema!{
    type Alien {
        name: String! @confql(arrayFilename: true)
        size: Float!
    }

    type Query {
        customers: [Alien!]!
    }

    schema {
        query: Query
    }
};

// We have some types generated to play with

let _mork = Alien {
    name: "Mork".to_string(),
    size: 12.0
};

// And juniper can execute, resolving against files

// Assemble some demo data on the filesystem
let mocks = TestFiles::new();
mocks.file(
    "customers/Paul.yml",
    indoc! {"
            ---
            size: 9.5
        "},
);

// The `Ctx` struct has been generated for us, implementing
// `juniper::Context`.  All it needs to initialize is a `PathBuf`
// pointing at the root of the data directory.
let ctx = Ctx::from(mocks.path().to_path_buf());

// Run the executor.
let (res, _errors) = juniper::execute_sync(
    indoc!{"
        query {
            customers {
                name
                size
            }
        }
    "},
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
        "customers": [
            {
                "name": "Paul",
                "size": 9.5
            }
        ]
    })
);
```

Current version: `0.4.0`

License: MIT
