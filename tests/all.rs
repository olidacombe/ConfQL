#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/graphql_schema_macro.rs");
}
