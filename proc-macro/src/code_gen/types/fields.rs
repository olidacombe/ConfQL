use graphql_parser::{query, schema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

pub struct Field<'doc, T: query::Text<'doc>> {
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

enum FieldType<'a, T: query::Text<'a>> {
    Nullable(query::Type<'a, T>),
    NonNullable(query::Type<'a, T>),
}

impl<'a, T> FieldType<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn inner_tokens(&self) -> TokenStream {
        use query::Type::{ListType, NamedType, NonNullType};
        match self.schema_type() {
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
    fn is_list(&self) -> bool {
        if let query::Type::ListType(_) = self.schema_type() {
            return true;
        }
        false
    }
    fn schema_type(&self) -> &schema::Type<'a, T> {
        use FieldType::{NonNullable, Nullable};
        let (Nullable(ty) | NonNullable(ty)) = self;
        ty
    }
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
