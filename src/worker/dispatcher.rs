use libc::c_void;
use std::cell::RefCell;
use std::rc::Rc;
use worker::{ffi, OpList, EntityId, ComponentId};

pub struct Dispatcher<T> {
	pointer: *const ffi::Worker_Dispatcher,
	data: Option<Rc<RefCell<T>>>,
	
	critical_section_cbs: Vec<Box<FnMut(Rc<RefCell<T>>, bool)>>,
	add_entity_cbs: Vec<Box<FnMut(Rc<RefCell<T>>, EntityId)>>,
	add_component_cbs: Vec<Box<FnMut(Rc<RefCell<T>>, EntityId, ComponentId, *const c_void)>>
}

impl<T> Drop for Dispatcher<T> {
	fn drop(&mut self) {
		unsafe {
			ffi::Worker_Dispatcher_Destroy(self.pointer);
		}
	}
}

impl<T> Dispatcher<T> {
	pub fn create() -> Box<Dispatcher<T>> {
		unsafe {
			let mut dispatcher = Box::new(Dispatcher {
				pointer: ffi::Worker_Dispatcher_Create(),
				data: None,
				
				critical_section_cbs: Vec::new(),
				add_entity_cbs: Vec::new(),
				add_component_cbs: Vec::new()
			});


			let dispatcher_ptr: *mut c_void = &mut (*dispatcher) as *mut _ as *mut c_void;
			ffi::Worker_Dispatcher_SetCriticalSectionCallback(dispatcher.pointer, 
				dispatcher_ptr,
				on_critical_section::<T>);
			ffi::Worker_Dispatcher_SetAddEntityCallback(dispatcher.pointer, 
				dispatcher_ptr,
				on_add_entity::<T>);
			ffi::Worker_Dispatcher_SetAddComponentCallback(dispatcher.pointer, 
				dispatcher_ptr,
				on_add_component::<T>);

			dispatcher
		}
	}

	pub fn set_data(&mut self, data: Rc<RefCell<T>>) {
		self.data = Some(data);
	}

	pub fn process(&self, op_list: OpList) {
		unsafe {
			ffi::Worker_Dispatcher_Process(self.pointer, op_list.pointer);
		}
	}

	pub fn register_critical_section_callback(&mut self, cb: Box<FnMut(Rc<RefCell<T>>, bool)>) {
		self.critical_section_cbs.push(cb);
	}

	pub fn register_add_entity_callback(&mut self, cb: Box<FnMut(Rc<RefCell<T>>, EntityId)>) {
		self.add_entity_cbs.push(cb);
	}

	pub fn register_add_component_callback(&mut self, cb: Box<FnMut(Rc<RefCell<T>>, EntityId, ComponentId, *const c_void)>) {
		self.add_component_cbs.push(cb);
	}

	fn on_critical_section(&mut self, op: *const ffi::Worker_CriticalSectionOp) {
		unsafe {
			let in_critical_section = (*op).in_critical_section == 1;
			match self.data {
				Some(ref data) => {
					for cb in self.critical_section_cbs.iter_mut() {
						(*cb)(data.clone(), in_critical_section);
					}
				}
				None => {}
			}
		}
	}

	fn on_add_entity(&mut self, op: *const ffi::Worker_AddEntityOp) {
		unsafe {
			let entity_id = (*op).entity_id;
			match self.data {
				Some(ref data) => {
					for cb in self.add_entity_cbs.iter_mut() {
						(*cb)(data.clone(), entity_id);
					}
				}
				None => {}
			}
			// let dispatcher_data: &mut T = &mut *(self.data.unwrap() as *mut T);
			// for cb in self.add_entity_cbs.iter_mut() {
			// 	(*cb)(self.data.unwrap(), entity_id);
			// }
		}
	}

	fn on_add_component(&mut self, op: *const ffi::Worker_AddComponentOp) {
		unsafe {
			let entity_id = (*op).entity_id;
			let component_id = (*op).data.component_id;
			let component_data = (*op).data.schema_type;
			match self.data {
				Some(ref data) => {
					for cb in self.add_component_cbs.iter_mut() {
						(*cb)(data.clone(), entity_id, component_id, component_data);
					}
				}
				None => {}
			}
			// let dispatcher_data: &mut T = &mut *(self.data.unwrap() as *mut T);
			// for cb in self.add_component_cbs.iter_mut() {
			// 	(*cb)(self.data.unwrap(), entity_id, component_id, data);
			// }
		}
	}
}

extern fn on_critical_section<T>(dispatcher: *mut c_void, op: *const ffi::Worker_CriticalSectionOp) {
	unsafe {
		(*(dispatcher as *mut Dispatcher<T>)).on_critical_section(op);
	}
}

extern fn on_add_entity<T>(dispatcher: *mut c_void, op: *const ffi::Worker_AddEntityOp) {
	unsafe {
		(*(dispatcher as *mut Dispatcher<T>)).on_add_entity(op);
	}
}

extern fn on_add_component<T>(dispatcher: *mut c_void, op: *const ffi::Worker_AddComponentOp) {
	unsafe {
		(*(dispatcher as *mut Dispatcher<T>)).on_add_component(op);
	}
}