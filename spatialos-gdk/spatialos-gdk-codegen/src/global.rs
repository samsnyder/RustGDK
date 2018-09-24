use user_type::UserType;
use component::Component;
use std::collections::HashMap;
use quote::Tokens;
use json::JsonCollection;
use std::rc::Rc;
use std::cell::RefCell;
use schema_type::Type;
use syn::Ident;

struct Module {
	sub_modules: HashMap<String, Module>,
	tokens: Vec<Tokens>
}

impl Module {
	pub fn new() -> Module {
		Module {
			sub_modules: HashMap::new(),
			tokens: Vec::new()
		}
	}

	pub fn add_tokens(&mut self, mut remaining_modules: Vec<String>, tokens: Tokens) {
		if remaining_modules.len() == 0 {
			self.tokens.push(tokens)
		}else{
			let next_module = self.sub_modules.entry(remaining_modules[0].clone()).or_insert(Module::new());
			remaining_modules.remove(0);
			next_module.add_tokens(remaining_modules, tokens);
		}
	}

	pub fn get_code(&self) -> Tokens {
		let sub_module_code = self.sub_modules.iter().map(|(name, module)| {
			let name = Ident::new(name.as_str());
			let module_code = module.get_code();
			quote!{
				#[allow(unused_parens)]
				pub mod #name {
					#[allow(unused_imports)] use spatialos_gdk::worker::{EntityId, ComponentId};
					#[allow(unused_imports)] use spatialos_gdk::worker::schema::{GeneratedSchema, Property, Event, Component, ComponentDataInterface, ComponentUpdateInterface,
																		Command, CommandRequestInterface, CommandResponseInterface};
					#[allow(unused_imports)] use spatialos_gdk::worker::ffi::{self, Schema_Object, Schema_ComponentData, Schema_ComponentUpdate, Schema_CommandRequest, Schema_CommandResponse};
					#[allow(unused_imports)] use schema::{Schema, ComponentData, ComponentUpdate};
					#[allow(unused_imports)] use std::collections::HashMap;
					#[allow(unused_imports)] use std::ops::Deref;

					#module_code
				}
			}
		});
		let tokens_code = self.tokens.iter();

		quote!{
			#(#sub_module_code)*

			#(#tokens_code)*
		}
	}
}

pub struct Global {
	types: HashMap<String, Rc<RefCell<UserType>>>,
	components: Vec<Component>
}

impl From<JsonCollection> for Global {
	fn from(value: JsonCollection) -> Global {
		let mut types: HashMap<String, Rc<RefCell<UserType>>> = value.typeDefinitions.into_iter().map(|type_def| {
				(type_def.qualifiedName.clone(), Rc::new(RefCell::new(UserType::from(type_def))))
			}).collect();

		let mut components: Vec<Component> = value.componentDefinitions.into_iter().map(|component_def| {
					Component::from(component_def)
				}).collect();

		for (_, mut user_type) in types.iter() {
			for mut field in user_type.borrow_mut().fields.iter_mut() {
				field.schema_type.got_all_types(&types);
			}
		}

		for component in components.iter_mut() {
			component.data_reference_type.got_all_types(&types);

			for mut event in component.events.iter_mut() {
				event.schema_type.got_all_types(&types);
				event.list_schema_type.got_all_types(&types);
			}

			for mut command in component.commands.iter_mut() {
				command.request_type.got_all_types(&types);
				command.response_type.got_all_types(&types);
			}
		}

		for component in components.iter() {
			types.remove(&component.data_reference_type.name);
		}

		Global {
			types,
			components
		}
	}
}

impl Global {
	pub fn get_code(&self) -> Tokens {
		let number_of_components = self.components.len();
		let dynamic_handler_code = self.components.iter().map(|component| {
			let language_qualified_name = Ident::new(component.rust_qualified_name.as_str());
			quote!(handler.register_component::<#language_qualified_name>())
		});

		let command_request_deserialise = self.components.iter().flat_map(|component| {
			component.commands.iter().map(|command| {
				command.get_global_request_match()
			})
		});
		let command_response_deserialise = self.components.iter().flat_map(|component| {
			component.commands.iter().map(|command| {
				command.get_global_response_match()
			})
		});
		let unique_index_match = self.components.iter().enumerate().map(|(index, component)| {
			let component_id = component.component_id;
			quote!(#component_id => Some(#index))
		});
		let data_enum_def = self.components.iter().map(|component| {
			let enum_name = Ident::new(component.enum_name().as_str());
			let data_qualified_name = Ident::new(format!("{}Data", component.rust_qualified_name));
			quote!(#enum_name(#data_qualified_name))
		});
		let update_enum_def = self.components.iter().map(|component| {
			let enum_name = Ident::new(component.enum_name().as_str());
			let data_qualified_name = Ident::new(format!("{}Update", component.rust_qualified_name));
			quote!(#enum_name(#data_qualified_name))
		});
		let data_deserialise = self.components.iter().map(|component| {
			let component_id = component.component_id;
			let data_qualified_name = Ident::new(format!("{}Data", component.rust_qualified_name));
			quote!(#component_id => Some(#data_qualified_name::deserialise_data(data)))
		});
		let data_serialise = self.components.iter().map(|component| {
			let enum_name = Ident::new(component.enum_name().as_str());
			quote!(&ComponentData::#enum_name(ref data) => data.serialise_data())
		});
		let data_apply_update = self.components.iter().map(|component| {
			let enum_name = Ident::new(component.enum_name().as_str());
			quote!{
				&mut ComponentData::#enum_name(ref mut data) => {
					if let &ComponentUpdate::#enum_name(ref update) = update {
						data.apply_update(&update);
					}
				}
			}
		});
		let update_deserialise = self.components.iter().map(|component| {
			let component_id = component.component_id;
			let update_qualified_name = Ident::new(format!("{}Update", component.rust_qualified_name));
			quote!(#component_id => Some(#update_qualified_name::deserialise_update(update)))
		});

		let mut module = Module::new();
		
		for user_type in self.types.values() {
			let user_type = user_type.borrow();
			let mut qualified_name = user_type.qualified_name.clone();
			qualified_name.pop();
			module.add_tokens(qualified_name, user_type.get_code());
		}

		for component in self.components.iter() {
			let mut qualified_name = component.qualified_name.clone();
			qualified_name.pop();
			module.add_tokens(qualified_name, component.get_code());
		}

		let module_code = module.get_code();

		quote!{
			#[allow(unused_variables)]
			pub mod schema {
				use spatialos_gdk::worker::schema::{Component, GeneratedSchema, GlobalComponentDataInterface,
					GlobalComponentUpdateInterface, ComponentDataInterface, ComponentUpdateInterface,
					DynamicComponentHandler};
				use spatialos_gdk::worker::ffi::{Schema_ComponentData, Schema_ComponentUpdate, 
					Schema_CommandResponse, Schema_CommandRequest};
				use spatialos_gdk::worker::{ComponentId};
				use std::any::Any;
				use spatialos_gdk::{ComponentBitField};
				use std::collections::HashMap;
				use spatialos_gdk;

				use schema::improbable::{EntityAcl, WorkerRequirementSet, WorkerAttributeSet};

				pub const GENERATED_NUMBER_OF_COMPONENTS: usize = #number_of_components;

				#[derive(Default)]
				pub struct Schema;

				#module_code

				impl GeneratedSchema for Schema {
				    const NUMBER_OF_COMPONENTS: usize = GENERATED_NUMBER_OF_COMPONENTS;
				    type ComponentData = ComponentData;
				    type ComponentUpdate = ComponentUpdate;
				    type ComponentBitField = GeneratedComponentBitField;

				    fn serialise_entity_acl(read: Vec<String>, write: HashMap<ComponentId, String>) -> (ComponentId, Box<Schema_ComponentData>) {
				    	let snapshot = EntityAcl {
				    		read_acl: WorkerRequirementSet {
				    			attribute_set: read.into_iter().map(|attribute| {
				                    WorkerAttributeSet {
				                        attribute: vec![attribute]
				                    }
				                }).collect()
				    		},
				    		component_write_acl: write.into_iter().map(|(id, attribute)| {
				    			let requirement_set = WorkerRequirementSet {
				    				attribute_set: vec![WorkerAttributeSet {
					    				attribute: vec![attribute]
					    			}]
				    			};
				    			(id, requirement_set)
				    		}).collect()
				    	};
				    	(EntityAcl::component_id(), snapshot.serialise_snapshot())
				    }

				    fn run_dynamic_component_handler<D: DynamicComponentHandler<Self>>(handler: &mut D) {
				        #(#dynamic_handler_code;)*
				    }

				    unsafe fn deserialise_command_request(
				        component_id: ComponentId,
				        command_index: u32,
				        request: Box<Schema_CommandRequest>,
				    ) -> Option<Box<Any>> {
				    	match (component_id, command_index) {
							#(#command_request_deserialise,)*
							_ => None
						}
				    }

				    unsafe fn deserialise_command_response(
				        component_id: ComponentId,
				        command_index: u32,
				        response: Box<Schema_CommandResponse>,
				    ) -> Option<Box<Any>> {
				    	match (component_id, command_index) {
							#(#command_response_deserialise,)*
							_ => None
						}
				    }
				}


				const GENERATED_NUMBER_OF_FIELDS: usize = (1 + ((GENERATED_NUMBER_OF_COMPONENTS - 1) / spatialos_gdk::FIELD_SIZE_BITS));

				#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Hash, Debug, Default)]
				pub struct GeneratedComponentBitField {
				    fields: [u64; GENERATED_NUMBER_OF_FIELDS]
				}

				impl ComponentBitField for GeneratedComponentBitField {
				    const NUMBER_OF_FIELDS: usize = GENERATED_NUMBER_OF_FIELDS;

				    fn new() -> Self {
				        GeneratedComponentBitField {
				            fields: [0; Self::NUMBER_OF_FIELDS],
				        }
				    }

				    fn get_field(&self, field_index: usize) -> &u64 {
				        &self.fields[field_index]
				    }
				    fn get_field_mut(&mut self, field_index: usize) -> &mut u64 {
				        &mut self.fields[field_index]
				    }

				    fn get_unique_index(component_id: ComponentId) -> Option<usize> {
				        match component_id {
							#(#unique_index_match,)*
							_ => None,
						}
				    }
				}


				pub enum ComponentData {
					#(#data_enum_def,)*
				}

				pub enum ComponentUpdate {
					#(#update_enum_def,)*
				}

				#[allow(dead_code, unused_variables)]
				impl GlobalComponentDataInterface<Schema> for ComponentData {
					fn deserialise(component_id: ComponentId, data: Box<Schema_ComponentData>) -> Option<ComponentData> {
						match component_id {
							#(#data_deserialise,)*
							_ => None
						}
					}

					fn serialise(&self) -> Box<Schema_ComponentData> {
						match self {
							#(#data_serialise,)*
						}
					}

					fn apply_update(&mut self, update: &ComponentUpdate){
						match self {
							#(#data_apply_update,)*
						}
					}
				}

				#[allow(dead_code, unused_variables)]
				impl GlobalComponentUpdateInterface<Schema> for ComponentUpdate {
					fn deserialise(component_id: ComponentId, update: Box<Schema_ComponentUpdate>) -> Option<ComponentUpdate> {
						match component_id {
							#(#update_deserialise,)*
							_ => None
						}
					}
				}
			}
		}
	}
}