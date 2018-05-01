use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use worker::{EntityId, ComponentId};
use worker::entity::Entity;
use worker::connection::Connection;
use worker::dispatcher::Dispatcher;
use worker::component::{ComponentUpdate};

pub struct View<T> {
	connection: Connection,
	dispatcher: Box<Dispatcher<View<T>>>,
	data: Rc<RefCell<T>>,
	view_data: Rc<RefCell<ViewData<T>>>
}

pub struct ViewData<T> {
	entities: HashMap<EntityId, Entity>,
	added_this_cs: Vec<EntityId>,
	add_entity_cbs: Vec<Box<Fn(&View<T>, Rc<RefCell<T>>, &Entity)>>,
	component_update_cbs: Vec<Box<Fn(&View<T>, Rc<RefCell<T>>, &Entity, &ComponentUpdate)>>,
}

impl<T> View<T> {
	pub fn new(connection: Connection, data: Rc<RefCell<T>>) -> Rc<RefCell<View<T>>> {
		let view_data = Rc::new(RefCell::new(ViewData {
			entities: HashMap::new(),
			added_this_cs: Vec::new(),
			add_entity_cbs: Vec::new(),
			component_update_cbs: Vec::new()
		}));

		let view = Rc::new(RefCell::new(View {
			connection,
			dispatcher: Dispatcher::create(),
			data,
			view_data
		}));
		

		// let view_ptr: *mut c_void = &mut *view as *mut _ as *mut c_void;
		view.borrow_mut().dispatcher.set_data(view.clone());

		view.borrow_mut().register_dispatcher_ops();

		view
	}

	pub fn process(&self) {
		let op_list = self.connection.get_op_list(1000);
    	self.dispatcher.process(op_list);
	}

	pub fn register_add_entity_callback(&self, cb: Box<Fn(&View<T>, Rc<RefCell<T>>, &Entity)>) {
		self.view_data.borrow_mut().add_entity_cbs.push(cb);
	}

	pub fn register_component_update_callback(&self, cb: Box<Fn(&View<T>, Rc<RefCell<T>>, &Entity, &ComponentUpdate)>) {
		self.view_data.borrow_mut().component_update_cbs.push(cb);
	}

	fn on_entity_added(&self, view_data: &ViewData<T>, entity_id: EntityId) {
		let entity =  view_data.entities.get(&entity_id).unwrap();
		for cb in view_data.add_entity_cbs.iter() {
			(*cb)(self, self.data.clone(), entity);
		}
	}

	fn on_component_update(&self, view_data: &ViewData<T>, entity_id: EntityId, component_id: ComponentId) {
		use schema::standard_library::PositionUpdate;
		let update = PositionUpdate::deserialise();
		let entity =  view_data.entities.get(&entity_id).unwrap();
		for cb in view_data.component_update_cbs.iter() {
			(*cb)(self, self.data.clone(), entity, &update);
		}
	}

	fn register_dispatcher_ops(&mut self) {
		self.dispatcher.register_critical_section_callback(Box::new(|view, in_critical_section| {
			if !in_critical_section {
				let view = view.borrow();
				let mut view_data = view.view_data.borrow_mut();

				for &entity_id in view_data.added_this_cs.iter() {
					view.on_entity_added(&view_data, entity_id);
				}
				view_data.added_this_cs.clear();
			}
		}));

		self.dispatcher.register_add_entity_callback(Box::new(|view, entity_id| {
			let view = view.borrow();
			let mut view_data = view.view_data.borrow_mut();
			let entity = Entity::new(entity_id);
			view_data.entities.insert(entity_id, entity);
			view_data.added_this_cs.push(entity_id);
		}));

		self.dispatcher.register_component_update_callback(Box::new(|view, entity_id, component_id| {
			let view = view.borrow();
			let mut view_data = view.view_data.borrow_mut();
			view.on_component_update(&view_data, entity_id, component_id);
		}));
	}
}