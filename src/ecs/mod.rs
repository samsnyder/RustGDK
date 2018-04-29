use specs::World;
use std::rc::Rc;
use std::cell::RefCell;
use worker::{View, Connection};
use schema::standard_library::Position;

pub struct EcsInterface {
	world: Rc<RefCell<World>>,
	view: Rc<RefCell<View<World>>>
}

impl EcsInterface {
	pub fn new(connection: Connection) -> EcsInterface {
		let world = Rc::new(RefCell::new(World::new()));
		
		world.borrow_mut().register::<Position>();

		let view = View::new(connection, world.clone());

		view.borrow_mut().register_add_entity_callback(Box::new(|_, world, entity| {
			println!("Entity ID {}", entity.entity_id);

			world.borrow_mut().create_entity().with(Position { x: 4.0, y: 7.0 }).build();
		}));

		EcsInterface {
			world,
			view
		}
	}

	pub fn process(&self) {
		self.view.borrow().process();
	}
}