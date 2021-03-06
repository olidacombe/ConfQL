use graphql_parser::{query, schema};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;

pub struct Field<'a, T: query::Text<'a>> {
    pub name: T::Value,
    field_type: FieldType<'a, T>,
    directives: HashMap<String, query::Value<'a, T>>,
}

impl<'a, T: query::Text<'a>> From<schema::Field<'a, T>> for Field<'a, T> {
    fn from(field: schema::Field<'a, T>) -> Self {
        let schema::Field {
            name,
            field_type,
            directives,
            ..
        } = field;
        let directives = directives
            .into_iter()
            .filter(|d| d.name.as_ref() == "confql")
            .flat_map(|d| d.arguments)
            .map(|(k, v)| (k.as_ref().to_owned(), v))
            .collect();
        Self {
            name,
            field_type: FieldType::from(field_type),
            directives,
        }
    }
}

impl<'a, T> Field<'a, T>
where
    T: query::Text<'a>,
{
    pub fn directive(&self, key: &str) -> Option<&query::Value<'a, T>> {
        self.directives.get(key)
    }
}

impl<'a, T> Field<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    pub fn merge_line(&self) -> TokenStream {
        let Field {
            name, field_type, ..
        } = self;
        let name = name.as_ref();
        let ty = field_type.inner_tokens();
        quote! {
            if let Ok(v) = <#ty>::resolve_value(data_path.join(#name)) {
                value.merge_at(#name, v)?;
            }
        }
    }
    pub fn resolver(&self) -> TokenStream {
        let Self {
            name, field_type, ..
        } = self;
        let name = name.as_ref();
        let field_name = format_ident!("{}", name);
        let getter = quote! {
            Ok(context.data_resolver.get(&[#name])?)
        };
        quote! {
            fn #field_name(context: &Ctx) -> FieldResult<#field_type> {
                #getter
            }
        }
    }
}

impl<'a, T> ToTokens for Field<'a, T>
where
    T: query::Text<'a>,
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Field {
            name, field_type, ..
        } = self;
        let name = format_ident!("{}", name.as_ref());
        tokens.extend(quote! { #name: #field_type });
    }
}

enum FieldType<'a, T: query::Text<'a>> {
    Nullable(query::Type<'a, T>),
    NonNullable(query::Type<'a, T>),
}

trait RustType {
    fn rust_type(&self) -> Ident;
}

impl RustType for &str {
    fn rust_type(&self) -> Ident {
        format_ident!(
            "{}",
            match self {
                &"Boolean" => "bool",
                &"Float" => "f64",
                &"ID" => "ID",
                &"Int" => "i32",
                v => v,
            }
        )
    }
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
                let val = val.as_ref().rust_type();
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

impl<'a, T> FieldType<'a, T>
where
    T: query::Text<'a>,
{
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
