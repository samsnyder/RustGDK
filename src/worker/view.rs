use std::collections::HashMap;
use std::cell::RefCell;
use worker::EntityId;
use worker::entity::Entity;
use worker::connection::Connection;
use worker::dispatcher::Dispatcher;
use libc::c_void;

pub struct View<'a, 'b: 'a, T: 'b> {
	connection: Connection,
	dispatcher: Box<Dispatcher<View<'a, 'b, T>>>,
	data: Option<&'b T>,

	entities: HashMap<EntityId, Entity>,
	added_this_cs: Vec<EntityId>,
	add_entity_cbs: Vec<Box<Fn(&Entity)>>,
}

impl<'a, 'b, T> View<'a, 'b, T> {
	pub fn new(connection: Connection) -> Box<View<'a, 'b, T>> {
		let mut view = Box::new(View {
			connection,
			dispatcher: Dispatcher::create(),
			data: None,
			entities: HashMap::new(),
			added_this_cs: Vec::new(),
			add_entity_cbs: Vec::new(),
		});
		

		let view_ptr: *mut c_void = &mut *view as *mut _ as *mut c_void;
		view.dispatcher.set_data(view_ptr);

		view.register_dispatcher_ops();

		view
	}

	pub fn set_data(&mut self, data: &'b T) {
		self.data = Some(data);
	}

	pub fn process(&self) {
		let op_list = self.connection.get_op_list(1000);
    	self.dispatcher.process(op_list);
	}

	pub fn register_add_entity_callback(&mut self, cb: Box<Fn(&Entity)>) {
		self.add_entity_cbs.push(cb);
	}

	fn on_entity_added(&self, entity_id: EntityId) {
		let entity = self.entities.get(&entity_id).unwrap();
		for cb in self.add_entity_cbs.iter() {
			(*cb)(entity);
		}
	}

	fn register_dispatcher_ops(&mut self) {
		self.dispatcher.register_critical_section_callback(Box::new(|view, in_critical_section| {
			if !in_critical_section {
				for &entity_id in view.added_this_cs.iter() {
					view.on_entity_added(entity_id);
				}
				view.added_this_cs.clear();
			}
		}));

		self.dispatcher.register_add_entity_callback(Box::new(|view, entity_id| {
			let entity = Entity::new(entity_id);
			view.entities.insert(entity_id, entity);
			view.added_this_cs.push(entity_id);
		}));
	}
}