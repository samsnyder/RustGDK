extern crate libc;
extern crate rand;
extern crate specs;

mod worker;
mod ecs;
mod schema;

use rand::Rng;
use ecs::EcsInterface;
use specs::{ReadStorage, System, World, DispatcherBuilder};
use specs::RunNow;
use schema::standard_library::Position;


struct HelloWorld;

impl<'a> System<'a> for HelloWorld {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, position: Self::SystemData) {
        use specs::Join;

        for position in position.join() {
            // println!("Hello, {:?}", &position);
        }
    }
}


fn main() {
	let mut worker_id = String::from("RustWorker");
	worker_id.push_str(rand::thread_rng().gen::<u16>().to_string().as_str());

	let conn = worker::Connection::connect_with_receptionist("UnityWorker", "127.0.0.1", 7777, worker_id.as_str());

	// let mut view = worker::View::new(conn);

	

	// ecs_interface.view.set_data(&ecs_interface);

	// view.register_add_entity_callback(Box::new(|entity| {
	// 	println!("Entity ID {}", entity.entity_id);

	// 	world.create_entity().with(Position { x: 4.0, y: 7.0 }).build();
	// }));

	let mut system = HelloWorld;
	
	let mut dispatcher = DispatcherBuilder::new()
	    .add(system, "hello_world", &[])
	    .build();

	let mut ecs_interface = EcsInterface::new(conn, dispatcher);

	// dispatcher.dispatch(&ecs_interface.world.borrow().res);

	// println!("dasdasdas");



	loop {
		ecs_interface.process();
	}
	// let mut disp = worker::Dispatcher::create();
	// disp.register_add_entity_callback(Box::new(|entity_id| {
	// 	println!("Entity ID {}", entity_id);
	// }));
	// disp.register_add_component_callback(Box::new(|entity_id, component_id, data| {
	// 	println!("Add component {} {}", entity_id, component_id);
	// }));
	// disp.register_critical_section_callback(Box::new(|in_critical_section| {
	// 	// println!("Critical {}", in_critical_section);
	// }));

 //    loop {
 //    	let op_list = conn.get_op_list(1000);
 //    	disp.process(op_list);
 //    	// println!("Op list");
 //    }
}
