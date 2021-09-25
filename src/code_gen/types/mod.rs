use graphql_parser::{query, schema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

mod fields;

use fields::Field;

pub struct Type<'doc, T: query::Text<'doc>> {
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

impl<'a, T: query::Text<'a>> From<schema::TypeDefinition<'a, T>> for Type<'a, T> {
    fn from(def: schema::TypeDefinition<'a, T>) -> Self {
        use schema::TypeDefinition;
        match def {
            TypeDefinition::Object(obj) => Self::from_object_definition(obj),
            _ => unimplemented! {},
        }
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
