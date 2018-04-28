extern crate libc;
extern crate rand;

mod worker;

use rand::Rng;
use worker::connection::Connection;

fn main() {
	let mut worker_id = String::from("RustWorker");
	worker_id.push_str(rand::thread_rng().gen::<u16>().to_string().as_str());

	let conn = Connection::connect_with_receptionist("UnityWorker", "127.0.0.1", 7777, worker_id.as_str());
	let mut disp = worker::Dispatcher::create();
	disp.register_add_entity_callback(Box::new(|entity_id| {
		println!("Entity ID {}", entity_id);
	}));
	disp.register_add_component_callback(Box::new(|entity_id, component_id, data| {
		println!("Add component {} {}", entity_id, component_id);
	}));
	disp.register_critical_section_callback(Box::new(|in_critical_section| {
		// println!("Critical {}", in_critical_section);
	}));

    loop {
    	let op_list = conn.get_op_list(1000);
    	disp.process(op_list);
    	// println!("Op list");
    }
}
