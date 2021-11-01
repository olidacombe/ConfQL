use graphql_parser::parse_schema;
use graphql_parser::{query, schema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::convert::TryFrom;
use thiserror::Error;

mod types;
use types::Type;

// https://nick.groenen.me/posts/rust-error-handling/#libraries-versus-applications
#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("No query definition in schema")]
    SchemaMissingQuery,
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
        let parsed = SchemaParse::<String>::try_from(parse_schema::<String>(&schema)?)?;
        Ok(parsed.into_token_stream())
    }
}

struct SchemaParse<'a, T: query::Text<'a>> {
    types: Vec<Type<'a, T>>,
    query_type: T::Value,
}

impl<'a, T> SchemaParse<'a, T>
where
    T: query::Text<'a>,
{
    fn imports(&self) -> TokenStream {
        quote! {
            use confql::confql_data_resolver::{DataPath, DataResolver, DataResolverError, Merge, ResolveValue};
            use juniper::{Context, FieldResult, GraphQLObject, ID, graphql_object};
            use serde::Deserialize;
        }
    }
    fn context(&self) -> TokenStream {
        quote! {
            struct Ctx {
                data_resolver: DataResolver
            }

            use std::path::PathBuf;
            impl Ctx {
                fn from<P: Into<PathBuf>>(p: P) -> Self {
                    Self {
                        data_resolver: DataResolver::from(p.into())
                    }
                }
            }

            impl juniper::Context for Ctx {}
        }
    }
    fn root_node(&self) -> TokenStream {
        let query_type = format_ident!("{}", self.query_type.as_ref());
        quote! {
            struct Mutation;

            type Schema = juniper::RootNode<'static, #query_type, juniper::EmptyMutation<Ctx>, juniper::EmptySubscription<Ctx>>;
        }
    }
}

impl<'a, T> ToTokens for SchemaParse<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.imports());
        tokens.extend(self.context());
        let types = self.types.iter();
        tokens.extend(quote! {#(#types)*});
        tokens.extend(self.root_node());
    }
}

impl<'a, T: query::Text<'a>> TryFrom<schema::Document<'a, T>> for SchemaParse<'a, T> {
    type Error = CodeGenError;

    fn try_from(doc: schema::Document<'a, T>) -> Result<Self, Self::Error> {
        use types::Object;
        let mut types = Vec::<Object<'a, T>>::new();
        let mut query_type: Option<T::Value> = None;

        use schema::Definition;
        doc.definitions.into_iter().for_each(|def| match def {
            Definition::TypeDefinition(def) => {
                types.push(Object::from(def));
            }
            Definition::SchemaDefinition(schema) => {
                if query_type.is_none() {
                    query_type = schema.query;
                }
            }
            _ => (),
        });

        if query_type.is_none() {
            return Err(Self::Error::SchemaMissingQuery);
        }
        let query_type = query_type.unwrap();
        let types = types
            .into_iter()
            .map(|t| {
                use Type::{Object, Query};
                if t.name == query_type {
                    Query(t)
                } else {
                    Object(t)
                }
            })
            .collect();

        Ok(Self { query_type, types })
    }
}
