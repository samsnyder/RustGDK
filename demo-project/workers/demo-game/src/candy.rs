// use rand::{self, Rng};
// use generated::Schema;
// use generated::demogame::{ExampleRequest, ExampleResponse, Transform};
// use generated::improbable::{Coordinates, Position, Vector3f};
// use spatialos_gdk::worker::schema::Component;
// use spatialos_gdk::{Entities, EntityBuilder, ModifiedRead, ModifiedWrite, Read, System, Worker,
//                     Write};
// use std::{thread, time};

// use spatialos_gdk::World;
// use spatialos_gdk::worker::EntityId;



// const CHECK_INTERVAL_FRAMES: usize = 20;
// const NUM_TO_REMOVE_PER_CHECK: usize = 5;
// const TARGET_NUMBER_OF_ENTITIES: usize = 20;
// const DIMENSION: f64 = 20.0;

// #[derive(ComponentGroup)]
// pub struct PersonData<'a> {
    
//     pub position: Write<'a, Schema, Position>,
//     pub entity_id: EntityId,
// }

// struct EntityMaintainerSystem {
// 	frames_since_check: usize
// }

// impl EntityMaintainerSystem {
// 	fn new() -> EntityMaintainerSystem {
// 		EntityMaintainerSystem {
// 			frames_since_check: 0
// 		}
// 	}

// 	fn create_entity(&self, world: &mut World<Schema>) {
//     	let position = Coordinates {
//             x: rand::thread_rng().gen::<f64>() * DIMENSION,
//             y: rand::thread_rng().gen::<f64>() * DIMENSION,
//             z: rand::thread_rng().gen::<f64>() * DIMENSION,
//         };
//     	world.create_entity(
//             EntityBuilder::new(vec![Worker::Type("rust")])
//                 .with_component(
//                     Worker::Type("rust"),
//                     Position {
//                         coords: position.clone(),
//                     },
//                 )
//                 .with_component(
//                     Worker::Type("rust"),
//                     Transform {
//                         position: Vector3f {
//                             x: position.x as f32,
//                             y: position.y as f32,
//                             z: position.z as f32,
//                         },
//                     },
//                 ),
//             |_world, entity_id| {
//                 println!("Created entity: {}", entity_id);
//             },
//             |_world, status, message| {
//                 println!(
//                     "Failure creating entity: {:?} {:?} {}",
//                     ::std::thread::current().id(),
//                     status,
//                     message
//                 );
//             },
//         );
//     }
// }

// impl System<Schema> for EntityMaintainerSystem {
//     fn on_update(&mut self, world: &mut World<Schema>, entities: &mut Entities<Schema>) {
//     	self.frames_since_check = self.frames_since_check + 1;

//     	if self.frames_since_check > CHECK_INTERVAL_FRAMES {
//     		self.frames_since_check = 0;

// 	        let entities: Vec<FlashingData> = entities.get::<FlashingData>().collect();

// 	        let num_to_remove = entities.len() - TARGET_NUMBER_OF_ENTITIES;
//         	for _ in (0..num_to_remove) {
//         		let id_to_remove = rand::thread_rng().choose(&entities).unwrap().entity_id;

//         		world.delete_entity(
// 		            id_to_remove,
// 		            |_world, entity_id| {
// 		                println!("delete success: {:?}", entity_id);
// 		            },
// 		            |_world, status, message| {
// 		                println!(
// 		                    "failure deleting entity: {:?} {:?} {}",
// 		                    ::std::thread::current().id(),
// 		                    status,
// 		                    message
// 		                );
// 		            },
// 		        );
//         	}

// 	        if entities.len() > TARGET_NUMBER_OF_ENTITIES {
	        	
// 	        }
// 	    }
//     }
// }