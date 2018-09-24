


use spatialos_gdk::worker::schema::{Component, GeneratedSchema, GlobalComponentDataInterface, GlobalComponentUpdateInterface, 
	ComponentDataInterface, ComponentUpdateInterface, DynamicComponentHandler, 
	CommandResponseInterface, CommandRequestInterface};
use spatialos_gdk::worker::ffi::{Schema_ComponentData, Schema_ComponentUpdate, Schema_CommandResponse, Schema_CommandRequest};
use spatialos_gdk::worker::{ComponentId};
use std::any::Any;
use spatialos_gdk::{self, ComponentBitField};
use std::collections::HashMap;

use generated;
use generated::improbable::{EntityAcl, WorkerRequirementSet, WorkerAttributeSet};

pub const GENERATED_NUMBER_OF_COMPONENTS: usize = {{NUMBER_OF_COMPONENTS}};

#[derive(Default)]
pub struct Schema;

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
        {{DYNAMIC_HANDLER_CODE}}
    }

    unsafe fn deserialise_command_request(
        component_id: ComponentId,
        command_index: u32,
        request: Box<Schema_CommandRequest>,
    ) -> Option<Box<Any>> {
    	match (component_id, command_index) {
			{{COMMAND_REQUEST_DESERIALISE_MATCH}}
			_ => None
		}
    }

    unsafe fn deserialise_command_response(
        component_id: ComponentId,
        command_index: u32,
        response: Box<Schema_CommandResponse>,
    ) -> Option<Box<Any>> {
    	match (component_id, command_index) {
			{{COMMAND_RESPONSE_DESERIALISE_MATCH}}
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
			{{UNIQUE_INDEX_MATCH}}
			_ => None,
		}
    }
}


pub enum ComponentData {
	{{DATA_ENUM_DEF}}
}

pub enum ComponentUpdate {
	{{UPDATE_ENUM_DEF}}
}

#[allow(dead_code, unused_variables)]
impl GlobalComponentDataInterface<Schema> for ComponentData {
	fn deserialise(component_id: ComponentId, data: Box<Schema_ComponentData>) -> Option<ComponentData> {
		match component_id {
			{{DATA_DESERIALISE}}
			_ => None
		}
	}

	fn serialise(&self) -> Box<Schema_ComponentData> {
		match self {
			{{DATA_SERIALISE}}
		}
	}

	fn apply_update(&mut self, update: &ComponentUpdate){
		match self {
			{{DATA_APPLY_UPDATE}}
		}
	}
}

#[allow(dead_code, unused_variables)]
impl GlobalComponentUpdateInterface<Schema> for ComponentUpdate {
	fn deserialise(component_id: ComponentId, update: Box<Schema_ComponentUpdate>) -> Option<ComponentUpdate> {
		match component_id {
			{{UPDATE_DESERIALISE}}
			_ => None
		}
	}
}

