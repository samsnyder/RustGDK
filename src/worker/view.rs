use std::collections::HashMap;
use worker::EntityId;
use worker::entity::Entity;
use worker::connection::Connection;
use worker::dispatcher::Dispatcher;
use libc::c_void;

pub struct View {
	connection: Connection,
	dispatcher: Box<Dispatcher<View>>,

	entities: HashMap<EntityId, Entity>
}

impl View {
	pub fn new(connection: Connection) -> Box<View> {
		let mut view = Box::new(View {
			connection,
			dispatcher: Dispatcher::create(),
			entities: HashMap::new()
		});
		

		let view_ptr: *mut c_void = &mut *view as *mut _ as *mut c_void;
		view.dispatcher.set_data(view_ptr);

		view.register_dispatcher_ops();

		view
	}

	pub fn process(&self) {
		let op_list = self.connection.get_op_list(1000);
    	self.dispatcher.process(op_list);
	}

	fn register_dispatcher_ops(&mut self) {
		self.dispatcher.register_add_entity_callback(Box::new(|view, entity_id| {
			let entity = Entity::new(entity_id);
			println!("New entity {}", entity_id);
			view.entities.insert(entity_id, entity);

			println!("New entityaaa {}", view.entities.len());
		}));
	}
}