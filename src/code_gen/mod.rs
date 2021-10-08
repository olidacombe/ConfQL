use graphql_parser::parse_schema;
use graphql_parser::{query, schema};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use thiserror::Error;

mod types;
use types::Type;

// https://nick.groenen.me/posts/rust-error-handling/#libraries-versus-applications
#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error(transparent)]
    QraphQLError(#[from] graphql_parser::schema::ParseError),
}

enum SchemaLocation {
    Literal(String),
}

pub struct CodeGen {
    source: SchemaLocation,
}

impl CodeGen {
    pub fn from_schema_literal(schema: String) -> Self {
        Self {
            source: SchemaLocation::Literal(schema),
        }
    }
    pub fn generate_code(self) -> Result<TokenStream, CodeGenError> {
        use SchemaLocation::Literal;
        // TODO a match when we have FilePath variant
        let Literal(schema) = self.source;
        let parsed = SchemaParse::<String>::from(parse_schema::<String>(&schema)?);
        Ok(parsed.into_token_stream())
    }
}

struct SchemaParse<'doc, T: query::Text<'doc>> {
    types: Vec<Type<'doc, T>>,
}

impl<'a, T> ToTokens for SchemaParse<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            use crate::data_resolver::{DataPath, DataResolverError, ResolveValue};
            use juniper::GraphQLObject;
            use serde::Deserialize;
        });
        let types = self.types.iter();
        tokens.extend(quote! {#(#types)*});
    }
}

impl<'a, T: query::Text<'a>> From<schema::Document<'a, T>> for SchemaParse<'a, T> {
    fn from(doc: schema::Document<'a, T>) -> Self {
        let mut types = Vec::<Type<'a, T>>::new();

        use schema::Definition;
        doc.definitions.into_iter().for_each(|def| {
            if let Definition::TypeDefinition(def) = def {
                types.push(Type::from(def))
            }
        });

        Self { types }
    }
}
