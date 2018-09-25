use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::{mem, ptr};
use worker::{ffi, ComponentId, Connection, EntityId};

pub struct SnapshotOutputStream {
    pointer: *mut ffi::Worker_SnapshotOutputStream,
    _vtable: Box<ffi::Worker_ComponentVtable>,
}

impl Drop for SnapshotOutputStream {
    fn drop(&mut self) {
        unsafe {
            ffi::Worker_SnapshotOutputStream_Destroy(self.pointer);
        }
    }
}

impl SnapshotOutputStream {
    pub fn new<P: AsRef<Path>>(filename: P) -> SnapshotOutputStream {
        let filename = CString::new(filename.as_ref().to_str().unwrap()).unwrap();
        let vtable = Connection::default_vtable();
        let vtable_ptr = Box::into_raw(vtable);
        let params = ffi::Worker_SnapshotParameters {
            component_vtable_count: 0,
            component_vtables: ptr::null(),
            default_component_vtable: vtable_ptr,
        };
        unsafe {
            let pointer = ffi::Worker_SnapshotOutputStream_Create(filename.as_ptr(), &params);

            SnapshotOutputStream {
                pointer,
                _vtable: Box::from_raw(vtable_ptr),
            }
        }
    }

    pub fn write_entity(
        &mut self,
        components: HashMap<ComponentId, Box<ffi::Schema_ComponentData>>,
        entity_id: EntityId,
    ) -> Result<(), &str> {
        unsafe {
            let mut components: Vec<ffi::Worker_ComponentData> = components
                .into_iter()
                .map(|(component_id, data)| {
                    let mut component_data: ffi::Worker_ComponentData = mem::zeroed();
                    component_data.component_id = component_id;
                    component_data.schema_type = Box::into_raw(data);
                    component_data
                })
                .collect();

            let components_ptr = components.as_mut_ptr();

            let entity = ffi::Worker_Entity {
                entity_id: entity_id,
                component_count: components.len() as u32,
                components: components_ptr,
            };
            let result = ffi::Worker_SnapshotOutputStream_WriteEntity(self.pointer, &entity);

            for component_data in components.iter() {
                Box::from_raw(component_data.schema_type);
            }

            if result == 0 {
                let error_ptr = ffi::Worker_SnapshotOutputStream_GetError(self.pointer);
                let error = CStr::from_ptr(error_ptr);
                Result::Err(error.to_str().unwrap())
            } else {
                Result::Ok(())
            }
        }
    }
}
