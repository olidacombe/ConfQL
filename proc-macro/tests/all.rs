#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/executable_schema.rs");
    t.pass("tests/file_name_as_array_field.rs");
    t.pass("tests/graphql_schema_macro.rs");
    t.pass("tests/happy_with_all_types.rs");
    t.pass("tests/queryable_schema.rs");
    t.pass("tests/renders_types_as_structs.rs");
}
