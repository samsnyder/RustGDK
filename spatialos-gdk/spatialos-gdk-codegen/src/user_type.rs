use field::Field;
use json::TypeDefinition;
use quote::Tokens;
use schema_type::Type;
use syn::Ident;
use to_rust_qualified_name;

pub struct UserType {
    name: String,
    pub qualified_name: Vec<String>,
    rust_qualified_name: String,
    pub fields: Vec<Field>,
}

impl UserType {
    pub fn get_code(&self) -> Tokens {
        let name = Ident::new(self.name.as_str());
        let field_definitions = self.fields
            .iter()
            .map(|field| field.definition_in_struct(false));
        let deserialise_fields = self.fields
            .iter()
            .map(|field| field.deserialise_into_struct());
        let serialise_fields = self.fields
            .iter()
            .map(|field| field.serialise_from_struct());

        quote!{
            #[allow(dead_code, unused_variables)]
            #[derive(Clone, Debug, Default)]
            pub struct #name {
                #(#field_definitions,)*
            }

            #[allow(dead_code, unused_variables)]
            impl #name {
                pub unsafe fn deserialise(object: *mut Schema_Object) -> #name {
                    #name {
                        #(#deserialise_fields,)*
                    }
                }

                pub unsafe fn serialise(&self, object: *mut Schema_Object) {
                    #(#serialise_fields;)*
                }
            }
        }
    }
}

impl Type for UserType {
    fn rust_type_name(&self) -> String {
        self.name.clone()
    }

    fn rust_qualified_name(&self) -> String {
        self.rust_qualified_name.clone()
    }

    fn deserialise_code(
        &self,
        object_name: &String,
        field_id: u32,
        index: Option<String>,
    ) -> Tokens {
        let object_name = Ident::new(object_name.as_str());
        let index = Ident::new(index.unwrap_or(String::from("0")).as_str());
        let rust_qualified_name = Ident::new(self.rust_qualified_name());
        quote!(#rust_qualified_name::deserialise(
            ffi::Schema_IndexObject(#object_name, #field_id, #index)))
    }

    fn serialise_code(&self, object_name: &String, field_id: u32, value_name: &String) -> Tokens {
        let object_name = Ident::new(object_name.as_str());
        let value_name = Ident::new(value_name.as_str());
        quote!(#value_name.serialise(ffi::Schema_AddObject(#object_name, #field_id)))
    }

    fn is_singular_type(&self) -> bool {
        true
    }
}

impl From<TypeDefinition> for UserType {
    fn from(value: TypeDefinition) -> UserType {
        UserType {
            name: value.name,
            qualified_name: value
                .qualifiedName
                .split(".")
                .map(|s| String::from(s))
                .collect(),
            rust_qualified_name: to_rust_qualified_name(value.qualifiedName.as_str()),
            fields: value
                .fieldDefinitions
                .into_iter()
                .map(|field_definition| field_definition.into())
                .collect(),
        }
    }
}
