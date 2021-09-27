use graphql_parser::{query, schema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

mod fields;

use fields::Field;

pub enum Type<'a, T: query::Text<'a>> {
    Object(Object<'a, T>),
}

pub struct Object<'doc, T: query::Text<'doc>> {
    name: T::Value,
    fields: Vec<Field<'doc, T>>,
}

impl<'doc, T: query::Text<'doc>> Type<'doc, T> {
    fn from_object_definition(def: schema::ObjectType<'doc, T>) -> Self {
        let fields = def.fields.into_iter().map(Field::from).collect();
        Self::Object(Object {
            name: def.name,
            fields,
        })
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
        tokens.extend(match self {
            Self::Object(obj) => {
                let name = format_ident!("{}", obj.name.as_ref());
                let fields = obj.fields.iter();
                quote! {
                    #[derive(GraphQLObject)]
                    struct #name {
                    #(#fields),*
                    }
                }
            }
        });
    }
}
