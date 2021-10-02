#![feature(specialization)]
extern crate proc_macro;

mod code_gen;
mod data_resolver;

use code_gen::CodeGen;

#[proc_macro]
pub fn graphql_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = input.to_string();

    let code_gen = CodeGen::from_schema_literal(schema);

    match code_gen.generate_code() {
        Ok(tokens) => tokens.into(),
        Err(errors) => panic!("{}", errors),
    }
}
