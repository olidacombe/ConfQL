extern crate proc_macro;

use quote::quote;

#[proc_macro]
pub fn graphql_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _schema = input.to_string();

    //     let code_gen = CodeGen::build_from_schema_literal(schema).finish();

    //     match code_gen.generate_code() {
    //         Ok(tokens) => tokens.into(),
    //         Err(errors) => panic!("{}", errors),
    //     }
    proc_macro::TokenStream::from(quote! {})
}
