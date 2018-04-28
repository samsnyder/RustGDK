use libc::c_void;
use worker::{ffi, OpList, EntityId, ComponentId};

pub struct Dispatcher {
	pointer: *const ffi::Worker_Dispatcher,
	
	critical_section_cbs: Vec<Box<FnMut(bool)>>,
	add_entity_cbs: Vec<Box<FnMut(EntityId)>>,
	add_component_cbs: Vec<Box<FnMut(EntityId, ComponentId, *const c_void)>>
}

impl Drop for Dispatcher {
	fn drop(&mut self) {
		unsafe {
			ffi::Worker_Dispatcher_Destroy(self.pointer);
		}
	}
}

impl Dispatcher {
	pub fn create() -> Box<Dispatcher> {
		unsafe {
			let mut dispatcher = Box::new(Dispatcher {
				pointer: ffi::Worker_Dispatcher_Create(),
				
				critical_section_cbs: Vec::new(),
				add_entity_cbs: Vec::new(),
				add_component_cbs: Vec::new()
			});

			ffi::Worker_Dispatcher_SetCriticalSectionCallback(dispatcher.pointer, 
				&mut (*dispatcher),
				on_critical_section);
			ffi::Worker_Dispatcher_SetAddEntityCallback(dispatcher.pointer, 
				&mut (*dispatcher),
				on_add_entity);
			ffi::Worker_Dispatcher_SetAddComponentCallback(dispatcher.pointer, 
				&mut (*dispatcher),
				on_add_component);

			dispatcher
		}
	}

	pub fn process(&self, op_list: OpList) {
		unsafe {
			ffi::Worker_Dispatcher_Process(self.pointer, op_list.pointer);
		}
	}

	pub fn register_critical_section_callback(&mut self, cb: Box<FnMut(bool)>) {
		self.critical_section_cbs.push(cb);
	}

	pub fn register_add_entity_callback(&mut self, cb: Box<FnMut(EntityId)>) {
		self.add_entity_cbs.push(cb);
	}

	pub fn register_add_component_callback(&mut self, cb: Box<FnMut(EntityId, ComponentId, *const c_void)>) {
		self.add_component_cbs.push(cb);
	}

	fn on_critical_section(&mut self, op: *const ffi::Worker_CriticalSectionOp) {
		unsafe {
			let in_critical_section = (*op).in_critical_section == 1;
			for cb in self.critical_section_cbs.iter_mut() {
				(*cb)(in_critical_section);
			}
		}
	}

	fn on_add_entity(&mut self, op: *const ffi::Worker_AddEntityOp) {
		unsafe {
			let entity_id = (*op).entity_id;
			for cb in self.add_entity_cbs.iter_mut() {
				(*cb)(entity_id);
			}
		}
	}

	fn on_add_component(&mut self, op: *const ffi::Worker_AddComponentOp) {
		unsafe {
			let entity_id = (*op).entity_id;
			let component_id = (*op).data.component_id;
			let data = (*op).data.schema_type;
			for cb in self.add_component_cbs.iter_mut() {
				(*cb)(entity_id, component_id, data);
			}
		}
	}
}

extern fn on_critical_section(dispatcher: *mut Dispatcher, op: *const ffi::Worker_CriticalSectionOp) {
	unsafe {
		(*dispatcher).on_critical_section(op);
	}
}

extern fn on_add_entity(dispatcher: *mut Dispatcher, op: *const ffi::Worker_AddEntityOp) {
	unsafe {
		(*dispatcher).on_add_entity(op);
	}
}

extern fn on_add_component(dispatcher: *mut Dispatcher, op: *const ffi::Worker_AddComponentOp) {
	unsafe {
		(*dispatcher).on_add_component(op);
	}
}