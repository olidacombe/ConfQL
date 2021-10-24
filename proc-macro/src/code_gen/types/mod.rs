use graphql_parser::{query, schema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

mod fields;

use fields::Field;

pub enum Type<'a, T: query::Text<'a>> {
    Object(Object<'a, T>),
    Query(Object<'a, T>),
}

pub struct Object<'a, T: query::Text<'a>> {
    pub name: T::Value,
    fields: Vec<Field<'a, T>>,
}

impl<'a, T: query::Text<'a>> Object<'a, T> {
    fn array_filename_fields(&self) -> Option<Vec<String>> {
        // TODO
        Some(vec!["noim".to_owned()])
    }
}

impl<'a, T: query::Text<'a>> From<schema::TypeDefinition<'a, T>> for Object<'a, T> {
    fn from(def: schema::TypeDefinition<'a, T>) -> Self {
        use schema::TypeDefinition;
        match def {
            TypeDefinition::Object(obj) => {
                let fields = obj.fields.into_iter().map(Field::from).collect();
                Self {
                    name: obj.name,
                    fields,
                }
            }
            _ => unimplemented! {},
        }
    }
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
                let merge_lines = obj.fields.iter().map(|f| f.merge_line());
                let mut resolve_value_methods = quote! {
                    fn merge_properties(
                        value: &mut serde_yaml::Value,
                        data_path: &DataPath
                    ) -> Result<(), DataResolverError> {
                        #(#merge_lines)*
                        Ok(())
                    }
                };
                if let Some(filename_fields) = obj.array_filename_fields() {
                    resolve_value_methods.extend(quote! {
                        fn resolve_vec_base(data_path: &DataPath) -> serde_yaml::Value {
                            use serde_yaml::{Mapping, Value};
                            let mut mapping = Mapping::new();
                            if let Some(file_stem) = data_path.file_stem() {
                                if let Some(file_stem) = file_stem.to_str() {
                                    for field in [#(#filename_fields),*] {
                                        mapping.insert(Value::from(field), Value::from(file_stem));
                                    }
                                }
                            }
                            Value::Mapping(mapping)
                        }
                    })
                }
                quote! {
                    #[derive(Deserialize)]
                    #[derive(GraphQLObject)]
                    struct #name {
                    #(#fields),*
                    }

                    impl ResolveValue for #name {
                        #resolve_value_methods
                    }
                }
            }
            Self::Query(obj) => {
                let name = format_ident!("{}", obj.name.as_ref());
                let resolvers = obj.fields.iter().map(|f| f.resolver());
                quote! {
                    struct #name;

                    #[graphql_object(context = Ctx)]
                    impl #name {
                        #(#resolvers)*
                    }
                }
            }
        });
    }
}
