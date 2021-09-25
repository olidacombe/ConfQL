use graphql_parser::parse_schema;
use graphql_parser::{query, schema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use thiserror::Error;

// https://nick.groenen.me/posts/rust-error-handling/#libraries-versus-applications
#[derive(Error, Debug)]
pub enum CodeGenError {
	#[error(transparent)]
	QraphQLError(#[from] graphql_parser::schema::ParseError),
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

enum SchemaLocation {
	Literal(String),
}

struct SchemaParse<'doc, T: query::Text<'doc>> {
	types: Vec<Type<'doc, T>>,
}

impl<'doc, T: query::Text<'doc>> ToTokens for SchemaParse<'doc, T> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
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

struct Type<'doc, T: query::Text<'doc>> {
	name: T::Value,
}

impl<'doc, T: query::Text<'doc>> ToTokens for Type<'doc, T> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let name = format_ident!("{}", self.name.as_ref());
		tokens.extend(quote! {
			struct #name {}
		});
	}
}

impl<'a, T: query::Text<'a>> From<schema::TypeDefinition<'a, T>> for Type<'a, T> {
	fn from(def: schema::TypeDefinition<'a, T>) -> Self {
		use schema::TypeDefinition;
		match def {
			TypeDefinition::Object(obj) => Self { name: obj.name },
			_ => unimplemented! {},
		}
	}
}
