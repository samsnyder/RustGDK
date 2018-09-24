use json::FieldDefinition;
use quote::Tokens;
use schema_type::{ListType, MapType, OptionType, Type};
use syn::Ident;

pub struct Field {
    name: String,
    field_id: u32,
    pub schema_type: Box<Type>,
}

impl From<FieldDefinition> for Field {
    fn from(value: FieldDefinition) -> Field {
        let schema_type: Box<Type> = if let Some(singular_type) = value.singularType {
            Box::<Type>::from(singular_type)
        } else if let Some(option_type) = value.optionType {
            Box::new(OptionType::from(option_type))
        } else if let Some(list_type) = value.listType {
            Box::new(ListType::from(list_type))
        } else if let Some(map_type) = value.mapType {
            Box::new(MapType::from(map_type))
        } else {
            panic!("Field definition is not a valid type: {:?}", value);
        };
        Field {
            name: value.name,
            field_id: value.number,
            schema_type: schema_type,
        }
    }
}

impl Field {
    pub fn definition_in_struct(&self, wrap_in_property: bool) -> Tokens {
        let type_name = if wrap_in_property {
            format!("Property<{}>", self.schema_type.rust_qualified_name())
        } else {
            self.schema_type.rust_qualified_name()
        };
        let type_name = Ident::new(type_name);
        let field_name = Ident::new(self.name.as_str());
        quote!(pub #field_name: #type_name)
    }

    pub fn definition_in_update(&self) -> Tokens {
        let type_name = Ident::new(self.schema_type.rust_qualified_name().as_str());
        let field_name = Ident::new(self.name.as_str());
        quote!(pub #field_name: Option<#type_name>)
    }

    pub fn apply_from_update(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        quote!{
            if let Some(ref value) = update.#field_name {
                self.#field_name = Property::new(value.clone());
            }
        }
    }

    pub fn deserialise_into_struct(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        let deserialise_code =
            self.schema_type
                .deserialise_code(&String::from("object"), self.field_id, None);
        quote!(#field_name: #deserialise_code)
    }

    pub fn serialise_from_struct(&self) -> Tokens {
        // let prefix = if self.schema_type.is_singular_type() || self.schema_type.is_built_in_type() {""}else{"*"};
        let value_name = format!("self.{}", &self.name);
        self.schema_type
            .serialise_code(&String::from("object"), self.field_id, &value_name)
    }

    pub fn deserialise_into_data(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        let deserialise_code =
            self.schema_type
                .deserialise_code(&String::from("object"), self.field_id, None);
        quote!(#field_name: Property::new(#deserialise_code))
    }

    pub fn serialise_from_data(&self) -> Tokens {
        let prefix = if self.schema_type.is_built_in_type() {
            "*"
        } else {
            ""
        };
        let value_name = format!("({}self.{}.deref())", prefix, &self.name);
        self.schema_type
            .serialise_code(&String::from("object"), self.field_id, &value_name)
    }

    pub fn deserialise_into_update(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        let count_code = self.schema_type
            .count_code(&String::from("fields"), self.field_id);
        let deserialise_code =
            self.schema_type
                .deserialise_code(&String::from("fields"), self.field_id, None);
        quote!(#field_name: {
			if #count_code > 0 {
				Some(#deserialise_code)
			}else{
				None
			}
		})
    }

    pub fn serialise_from_dirty_data(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        let prefix = if self.schema_type.is_built_in_type() {
            "*"
        } else {
            ""
        };
        let value_name = format!("({}self.{}.deref())", prefix, &self.name);
        let serialise_code = self.schema_type.serialise_code(
            &String::from("fields"),
            self.field_id,
            &String::from(value_name),
        );
        quote!{
            if self.#field_name.get_and_clear_dirty_bit() {
                #serialise_code;
            }
        }
    }

    pub fn is_dirty(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        quote!(self.#field_name.get_dirty_bit() ||)
    }

    pub fn snapshot_to_data(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        quote!(#field_name: self.#field_name.into())
    }
}
