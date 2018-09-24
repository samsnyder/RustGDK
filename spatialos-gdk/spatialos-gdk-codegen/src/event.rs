use json::{EventDefinition, ListTypeDefinition};
use quote::Tokens;
use schema_type::Type;
use schema_type::{ListType, ReferencedUserType};
use syn::Ident;

pub struct Event {
    name: String,
    event_index: u32,
    pub schema_type: ReferencedUserType,
    pub list_schema_type: ListType,
}

impl From<EventDefinition> for Event {
    fn from(value: EventDefinition) -> Event {
        Event {
            name: value.name,
            event_index: value.eventIndex,
            schema_type: ReferencedUserType::from(value.eventType.clone()),
            list_schema_type: ListType::from(ListTypeDefinition {
                valueType: value.eventType,
            }),
        }
    }
}

impl Event {
    pub fn definition_in_struct(&self) -> Tokens {
        let type_name = Ident::new(self.schema_type.rust_qualified_name().as_str());
        let field_name = Ident::new(self.name.as_str());
        quote!(pub #field_name: Event<#type_name>)
    }

    pub fn definition_in_update(&self) -> Tokens {
        let type_name = Ident::new(self.schema_type.rust_qualified_name().as_str());
        let field_name = Ident::new(self.name.as_str());
        quote!(pub #field_name: Vec<#type_name>)
    }

    pub fn apply_from_update(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        quote!{
            for value in update.#field_name.iter() {
                self.#field_name.add_event(value.clone());
            }
        }
    }

    pub fn deserialise_into_update(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        let deserialise_code = self.list_schema_type.deserialise_code(
            &String::from("events_obj"),
            self.event_index,
            None,
        );
        quote!(#field_name: {
			let events_obj = ffi::Schema_GetComponentUpdateEvents(update);
			#deserialise_code
		})
    }

    pub fn serialise_from_dirty_data(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        let value_name = format!("self.{}.get_staged_events()", field_name);
        let serialise_code = self.list_schema_type.serialise_code(
            &String::from("events"),
            self.event_index,
            &value_name,
        );
        quote!{
            #serialise_code;
            self.#field_name.clear_staged_events();
        }
    }

    pub fn initial_code(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        quote!(#field_name: Event::new())
    }

    pub fn contains_events_code(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        quote!(self.#field_name.len() > 0)
    }

    pub fn clear_events_code(&self) -> Tokens {
        let field_name = Ident::new(self.name.as_str());
        quote!(self.#field_name.clear())
    }
}
