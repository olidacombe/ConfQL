#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/graphql_schema_macro.rs");
    t.pass("tests/renders_simple_types_as_structs");
}
