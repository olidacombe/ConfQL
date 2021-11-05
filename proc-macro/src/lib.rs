extern crate proc_macro;

mod code_gen;

use code_gen::CodeGen;
use std::path::PathBuf;

fn generate_code(gen: CodeGen) -> proc_macro::TokenStream {
    match gen.generate_code() {
        Ok(tokens) => tokens.into(),
        Err(errors) => panic!("{}", errors),
    }
}

#[proc_macro]
pub fn graphql_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = input.to_string();
    generate_code(CodeGen::from_schema_literal(schema))
}

#[proc_macro]
pub fn graphql_schema_from_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema_path: PathBuf = input.to_string().into();
    let cargo_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Env var `CARGO_MANIFEST_DIR` was missing");
    let pwd = PathBuf::from(cargo_dir);
    let schema_path = pwd.join(schema_path);
    generate_code(CodeGen::from_schema_file(schema_path))
}
