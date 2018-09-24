#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Body, DeriveInput, Field, Ident, PathParameters, Ty, VariantData};

#[proc_macro_derive(ComponentGroup)]
pub fn component_group(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_component_group(&ast);
    gen.parse().unwrap()
}

enum GroupField {
    Component(ComponentField),
    EntityId(Ident),
}

impl GroupField {
    fn from_field(field: &Field) -> GroupField {
        let requirement = match field.ty {
            Ty::Path(_, ref path) if path.segments.len() > 0 => path.segments.last().unwrap(),
            _ => panic!("All fields must be component types"),
        }.clone();

        match requirement.ident.as_ref() {
            "EntityId" => GroupField::EntityId(field.ident.clone().unwrap()),
            _ => GroupField::Component(ComponentField::from_field(field)),
        }
    }

    fn get_index_storage_code(&self) -> Tokens {
        match self {
            GroupField::Component(component) => component.get_index_storage_code(),
            GroupField::EntityId(field_name) => {
                quote!(#field_name: (*chunk_ptr.0).get_entity_id(_index))
            }
        }
    }

    fn get_sendable_index_storage_code(&self) -> Tokens {
        match self {
            GroupField::Component(component) => component.get_sendable_index_storage_code(),
            GroupField::EntityId(field_name) => {
                quote!(#field_name: (*chunk_ptr.0).get_entity_id(_index))
            }
        }
    }
}

struct ComponentField {
    field_name: Ident,
    component: Ident,
    requirement: Ident,
}

impl ComponentField {
    fn from_field(field: &Field) -> ComponentField {
        let requirement = match field.ty {
            Ty::Path(_, ref path) if path.segments.len() > 0 => path.segments.last().unwrap(),
            _ => panic!("All fields must be component types"),
        }.clone();
        let component = match requirement.parameters {
            PathParameters::AngleBracketed(ref parameter_data) => match &parameter_data.types[1] {
                Ty::Path(_, ref path) if path.segments.len() > 0 => {
                    &path.segments.last().unwrap().ident
                }
                _ => panic!("All fields must be component types"),
            },
            _ => panic!("All fields must be component types"),
        }.clone();

        ComponentField {
            field_name: field.ident.clone().unwrap(),
            component,
            requirement: requirement.ident,
        }
    }

    fn get_authority_filter_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        match self.requirement.as_ref() {
            "Write" | "ModifiedWrite" => quote!((*#storage_name).get_authority(*_index) != 
                    ::spatialos_gdk::worker::Authority::NotAuthoritative),
            _ => quote!(true),
        }
    }

    fn get_sendable_authority_filter_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        match self.requirement.as_ref() {
            "Write" | "ModifiedWrite" => quote!((*#storage_name.0).get_authority(_index) != 
                    ::spatialos_gdk::worker::Authority::NotAuthoritative),
            _ => quote!(true),
        }
    }

    fn get_last_updated_filter_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        match self.requirement.as_ref() {
            "ModifiedRead" | "ModifiedWrite" => {
                quote!((*#storage_name).get_component_data_entry(*_index).last_updated
                    .occured_after(_from_time))
            }
            _ => quote!(true),
        }
    }

    fn get_sendable_last_updated_filter_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        match self.requirement.as_ref() {
            "ModifiedRead" | "ModifiedWrite" => {
                quote!((*#storage_name.0).get_component_data_entry(_index).last_updated
                    .occured_after(_from_time))
            }
            _ => quote!(true),
        }
    }

    fn get_chunk_storage_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        let component = &self.component;
        quote!(let #storage_name = chunk.get_component_storage::<#component>().unwrap()
            as *mut ::spatialos_gdk::ComponentStorage<::schema::Schema, #component>)
    }

    fn get_sendable_chunk_storage_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        let component = &self.component;
        quote!(let #storage_name = ::spatialos_gdk::UnsafeSendablePointer(
            chunk.get_component_storage::<#component>().unwrap()
            as *mut ::spatialos_gdk::ComponentStorage<::schema::Schema, #component>
        ))
    }

    fn get_chunk_storage_dirty_code(&self) -> Tokens {
        let component = &self.component;
        match self.requirement.as_ref() {
            "Write" | "ModifiedWrite" => {
                quote!(chunk.mark_component_storage_as_dirty::<#component>();)
            }
            _ => quote!(),
        }
    }

    fn get_index_storage_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        let field_name = &self.field_name;
        match self.requirement.as_ref() {
            "Read" | "ModifiedRead" => quote!(#field_name: Read::new(&(*#storage_name)
                    .get_component_data_entry(_index).data)),
            "Write" | "ModifiedWrite" => quote!(#field_name: Write::new(&mut (*#storage_name)
                    .get_component_data_entry(_index).data)),
            _ => panic!("All fields must be component types"),
        }
    }

    fn get_sendable_index_storage_code(&self) -> Tokens {
        let storage_name = Ident::new(format!("storage{}", self.component));
        let field_name = &self.field_name;
        match self.requirement.as_ref() {
            "Read" | "ModifiedRead" => quote!(#field_name: Read::new(&(*#storage_name.0)
                    .get_component_data_entry(_index).data)),
            "Write" | "ModifiedWrite" => quote!(#field_name: Write::new(&mut (*#storage_name.0)
                    .get_component_data_entry(_index).data)),
            _ => panic!("All fields must be component types"),
        }
    }
}

fn impl_component_group(input: &DeriveInput) -> Tokens {
    let input_type = &input.ident;
    let fields = match input.body {
        Body::Struct(VariantData::Struct(ref fields)) => {
            fields.iter().map(|field| GroupField::from_field(field))
        }
        _ => panic!("Only structs can derive ComponentGroup"),
    };

    let component_fields = fields.clone().filter_map(|field| match field {
        GroupField::Component(c) => Some(c),
        _ => None,
    });
    let component_names = component_fields
        .clone()
        .map(|field| field.component.clone());
    let authority_filter_code = component_fields
        .clone()
        .map(|field| field.get_authority_filter_code());
    let last_updated_filter_code = component_fields
        .clone()
        .map(|field| field.get_last_updated_filter_code());
    let chunk_storage_code = component_fields
        .clone()
        .map(|field| field.get_chunk_storage_code());
    let chunk_storage_dirty_code = component_fields
        .clone()
        .map(|field| field.get_chunk_storage_dirty_code());
    let index_storage_code = fields.clone().map(|field| field.get_index_storage_code());

    let sendable_chunk_storage_code = component_fields
        .clone()
        .map(|field| field.get_sendable_chunk_storage_code());
    let chunk_storage_dirty_code_clone = chunk_storage_dirty_code.clone();
    let sendable_authority_filter_code = component_fields
        .clone()
        .map(|field| field.get_sendable_authority_filter_code());
    let sendable_last_updated_filter_code = component_fields
        .clone()
        .map(|field| field.get_sendable_last_updated_filter_code());
    let sendable_index_storage_code = fields
        .clone()
        .map(|field| field.get_sendable_index_storage_code());

    quote!{
        #[allow(non_snake_case,unused_unsafe,unused_imports)]
        impl<'a> ::spatialos_gdk::ComponentGroup<'a, ::schema::Schema> for #input_type<'a> {
            fn add_to_bit_field(bit_field: &mut <::schema::Schema as
                ::spatialos_gdk::worker::schema::GeneratedSchema>::ComponentBitField) {
                use ::spatialos_gdk::ComponentBitField;
                use ::spatialos_gdk::worker::schema::Component;

                #(bit_field.add_component(#component_names::component_id());)*
            }

            fn get_iterator(chunk: &'a mut ::spatialos_gdk::Chunk<::schema::Schema>,
                _from_time: &'a ::spatialos_gdk::WorldTime) -> Box<Iterator<Item = Self> + 'a> {
                use ::spatialos_gdk::worker::schema::Component;

                let chunk_ptr: ::spatialos_gdk::UnsafeSendablePointer
                    <::spatialos_gdk::Chunk<::schema::Schema>> =
                    ::spatialos_gdk::UnsafeSendablePointer(chunk
                        as *mut ::spatialos_gdk::Chunk<::schema::Schema>);

                #(#chunk_storage_code;)*
                #(#chunk_storage_dirty_code;)*

                Box::new(
                    chunk.entity_index_iter().filter(move |_index| {
                        unsafe {
                            #(#authority_filter_code) && *
                            && #(#last_updated_filter_code) && *
                        }
                    }).map(move |_index| {
                        unsafe {
                            #input_type {
                                #(#index_storage_code),*
                            }
                        }
                    })
                )
            }

            fn par_for_each<F: Send + Sync>(chunk: &'a mut ::spatialos_gdk::Chunk<::schema::Schema>,
                _from_time: &'a ::spatialos_gdk::WorldTime, cb: F)
                where F: Fn(&mut Self)
            {
                use ::spatialos_gdk::worker::schema::Component;

                let chunk_ptr: ::spatialos_gdk::UnsafeSendablePointer
                    <::spatialos_gdk::Chunk<::schema::Schema>> =
                    ::spatialos_gdk::UnsafeSendablePointer(chunk
                        as *mut ::spatialos_gdk::Chunk<::schema::Schema>);

                #(#sendable_chunk_storage_code;)*
                #(#chunk_storage_dirty_code_clone;)*

                chunk.par_for_each_entity_index(|_index| {
                    unsafe {
                        if #(#sendable_authority_filter_code) && *
                            && #(#sendable_last_updated_filter_code) && * {
                            let mut value = #input_type {
                                #(#sendable_index_storage_code),*
                            };
                            cb(&mut value);
                        }
                    }
                });
            }
        }
    }
}
