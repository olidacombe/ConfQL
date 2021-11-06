//! Some procedural macros taking a GraphQL schema, and generating
//! structs for all types in the schema with data resolution impls,
//! along with a [juniper::RootNode] ready for execution.
#![deny(missing_docs)]
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

/// Generates a [juniper::RootNode] and structs with data resolution
/// impls from a literal schema in code.
/// This is predominantly used for tests.  Most likely,
/// clients of this crate would instead use [graphql_schema_from_file!].
///
/// # Example
///
/// ```ignore
/// // You should use the re-exported version of this from
/// // the `confql` crate, where the necessary dependencies
/// // exist for the generated code.
///
/// graphql_schema!{
///     type Job {
///         name: String!
///         easy: Bool!
///     }
///
///     type Query {
///         jobs: [Job!]!
///     }
///
///     schema {
///         query: Query
///     }
/// };
/// ```
#[proc_macro]
pub fn graphql_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = input.to_string();
    generate_code(CodeGen::from_schema_literal(schema))
}

/// Generates a [juniper::RootNode] and structs with data resolution impls from a
/// schema file path relative to the callee's crate root.
///
/// # Example
///
/// ```ignore
/// // You should use the re-exported version of this from
/// // the `confql` crate, where the necessary dependencies
/// // exist for the generated code.
///
/// // No quotes in file name!
/// graphql_schema_from_file!(schema.gql);
/// ```
#[proc_macro]
pub fn graphql_schema_from_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema_path: PathBuf = input.to_string().into();
    let cargo_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Env var `CARGO_MANIFEST_DIR` was missing");
    let pwd = PathBuf::from(cargo_dir);
    let schema_path = pwd.join(schema_path);
    generate_code(CodeGen::from_schema_file(schema_path))
}
