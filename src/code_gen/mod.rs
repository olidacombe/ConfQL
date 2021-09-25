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

impl<'a, T> ToTokens for SchemaParse<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
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

enum FieldType<'a, T: query::Text<'a>> {
    Nullable(query::Type<'a, T>),
    NonNullable(query::Type<'a, T>),
}

impl<'a, T: query::Text<'a>> From<schema::Type<'a, T>> for FieldType<'a, T> {
    fn from(ty: schema::Type<'a, T>) -> Self {
        use query::Type::NonNullType;
        match ty {
            NonNullType(t) => Self::NonNullable(*t),
            _ => Self::Nullable(ty),
        }
    }
}

impl<'a, T> FieldType<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn inner_tokens(&self) -> TokenStream {
        use query::Type::{ListType, NamedType, NonNullType};
        let ty = match self {
            Self::Nullable(ty) => ty,
            Self::NonNullable(ty) => ty,
        };
        match ty {
            NamedType(val) => {
                let val = format_ident!("{}", val.as_ref());
                quote! {#val}
            }
            ListType(t) => {
                let t = Self::from(*t.clone());
                quote! { Vec<#t> }
            }
            NonNullType(_) => unreachable!(),
        }
    }
}

struct Field<'doc, T: query::Text<'doc>> {
    name: T::Value,
    field_type: FieldType<'doc, T>,
}

impl<'a, T: query::Text<'a>> From<schema::Field<'a, T>> for Field<'a, T> {
    fn from(field: schema::Field<'a, T>) -> Self {
        let schema::Field {
            name, field_type, ..
        } = field;
        Self {
            name,
            field_type: FieldType::from(field_type),
        }
    }
}

struct Type<'doc, T: query::Text<'doc>> {
    name: T::Value,
    fields: Vec<Field<'doc, T>>,
}

impl<'doc, T: query::Text<'doc>> Type<'doc, T> {
    fn from_object_definition(def: schema::ObjectType<'doc, T>) -> Self {
        let fields = def.fields.into_iter().map(Field::from).collect();
        Self {
            name: def.name,
            fields,
        }
    }
}

impl<'a, T> ToTokens for FieldType<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner_type = self.inner_tokens();
        tokens.extend(match self {
            Self::Nullable(_) => quote! { Option<#inner_type>},
            Self::NonNullable(_) => quote! { #inner_type },
        });
    }
}

impl<'a, T> ToTokens for Field<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Field { name, field_type } = self;
        let name = format_ident!("{}", name.as_ref());
        tokens.extend(quote! { #name: #field_type });
    }
}

impl<'a, T> ToTokens for Type<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name.as_ref());
        let fields = self.fields.iter();
        tokens.extend(quote! {
            struct #name {
                #(#fields),*
            }
        });
    }
}

impl<'a, T: query::Text<'a>> From<schema::TypeDefinition<'a, T>> for Type<'a, T> {
    fn from(def: schema::TypeDefinition<'a, T>) -> Self {
        use schema::TypeDefinition;
        match def {
            TypeDefinition::Object(obj) => Self::from_object_definition(obj),
            _ => unimplemented! {},
        }
    }
}
