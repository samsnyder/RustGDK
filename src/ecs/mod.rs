use specs::World;
use std::rc::Rc;
use worker::{View, Connection};
use schema::standard_library::Position;

pub struct EcsInterface<'a> {
	world: World,
	view: Box<View<'a, 'a, EcsInterface<'a>>>
}

impl<'a> EcsInterface<'a> {
	pub fn run(connection: Connection) {

		let mut view = View::new(connection);
		let mut world = World::new();
		world.register::<Position>();

		// view.register_add_entity_callback(Box::new(|entity| {
		// 	println!("Entity ID {}", entity.entity_id);

		// 	world.create_entity().with(Position { x: 4.0, y: 7.0 }).build();
		// }));

		let mut ecs_interface = Box::new(EcsInterface {
			world,
			view
		});

		{
			ecs_interface.view.set_data(&ecs_interface);
		}

		// ecs_interface
	}

	pub fn process(&self) {
		self.view.process();
	}
}