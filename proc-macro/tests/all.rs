#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/executable_schema.rs");
    t.pass("tests/graphql_schema_macro.rs");
    t.pass("tests/renders_types_as_structs.rs");
}
