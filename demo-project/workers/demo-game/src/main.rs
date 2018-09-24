extern crate rand;
extern crate spatialos_gdk;
#[macro_use]
extern crate spatialos_gdk_derive;

extern crate spatialos_gdk_codegen;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

mod flashing;
// mod generated;

use schema::Schema;
use schema::demogame::{ExampleRequest, ExampleResponse, Transform};
use schema::improbable::{Coordinates, Position, Vector3f};
use rand::Rng;
use spatialos_gdk::worker::schema::Component;
use spatialos_gdk::{Entities, EntityBuilder, ModifiedRead, ModifiedWrite, Read, System, Worker,
                    Write};
use std::{thread, time};

use spatialos_gdk::World;
use spatialos_gdk::worker::EntityId;

#[derive(ComponentGroup)]
pub struct PlayerData<'a> {
    // pub transform: ModifiedWrite<'a, Schema, Transform>,
    pub position: Write<'a, Schema, Position>,
    pub entity_id: EntityId,
}

struct TestSystem {}

impl System<Schema> for TestSystem {
    fn on_ready(&mut self, world: &mut World<Schema>) {
        world.register_command_handler(
            Transform::example_command(),
            |world, entity_id, request| {
                println!("Got request: {} {:?}", entity_id, request);

                let mut position = world.get_mut_component::<Position>(entity_id).unwrap();
                position.coords.x = 3.0;
                println!("{:?}", position.coords);

                ExampleResponse { reply: 0.1 }
            },
        );

        world.delete_entity(
            100,
            |_world, entity_id| {
                println!("delete success: {:?}", entity_id);
            },
            |_world, status, message| {
                println!(
                    "failure deleting entity: {:?} {:?} {}",
                    ::std::thread::current().id(),
                    status,
                    message
                );
            },
        );

        world.send_command(
            Transform::example_command(),
            5,
            ExampleRequest { param: 0.5 },
            |_world, entity_id, response| {
                println!("command success: {} {:?}", entity_id, response.reply);
            },
            |_world, status, message| {
                println!(
                    "failure: {:?} {:?} {}",
                    ::std::thread::current().id(),
                    status,
                    message
                );
            },
        );

        world.create_entity(
            EntityBuilder::new(vec![Worker::Type("managed"), Worker::Type("rust")])
                .with_component(
                    Worker::Type("rust"),
                    Position {
                        coords: Coordinates {
                            x: 0.1,
                            y: 0.2,
                            z: 0.3,
                        },
                    },
                )
                .with_component(
                    Worker::Type("rust"),
                    Transform {
                        position: Vector3f {
                            x: 0.1,
                            y: 0.2,
                            z: 0.3,
                        },
                    },
                ),
            |_world, entity_id| {
                println!("Created entity: {}", entity_id);
            },
            |_world, status, message| {
                println!(
                    "failure creating entity: {:?} {:?} {}",
                    ::std::thread::current().id(),
                    status,
                    message
                );
            },
        );
    }

    fn on_update(&mut self, world: &mut World<Schema>, entities: &mut Entities<Schema>) {
        println!("\n\nLoop1");

        entities.par_for_each::<PlayerData, _>(|player| {
            // if rand::thread_rng().gen::<u8>() < 20 {
            player.position.coords.x = rand::thread_rng().gen::<f64>().into();
            // }

            println!(
                "{:?} {} {:?}",
                ::std::thread::current().id(),
                player.entity_id,
                player.position.coords
            );
        });

        // let player_list = world.get_shared_resource::<PlayerList>().unwrap();

        // for mut player in entities.get::<PlayerData>() {
        //     if player_list.players.contains(&player.entity_id) {
        //         println!("{} is a player", player.entity_id);
        //     }
        // }

        // for mut player in entities.get::<PlayerData>() {
        //     // if rand::thread_rng().gen::<u8>() < 20 {
        //     //     player.transform.position.x = rand::thread_rng().gen::<f32>().into();
        //     // }

        //     // player.position.coords.x = rand::thread_rng().gen::<f32>().into();

        //     // let transform = world.get_component::<Transform>(player.entity_id).unwrap();

        //     println!(
        //         "{:?} {} {:?}",
        //         ::std::thread::current().id(),
        //         player.entity_id,
        //         player.position.coords
        //     );

        //     // world.delete_entity(player.entity_id, |entity_id| {
        //     //     println!("delete success: {:?}", entity_id);
        //     // }, |status, message| {
        //     //     println!("failure: {:?} {}", status, message);
        //     // });

        //     // player.transform.update_rotation.trigger(RotationEvent {
        //     //     angle: rand::thread_rng().gen::<f32>().into(),
        //     // });

        //     // for rot in &player.transform.update_rotation {
        //     //     println!("{:?}", rot.angle);
        //     // }

        //     // for pos in &player.position.add_health {
        //     //     println!("{:?}", pos);

        //     //     player.position.coords.x = 5.0;
        //     // player.controls.target_speed = 2.0.into();

        //     //     player.position.coords = Coordinates {
        //     //         x: 2.0,
        //     //         y: 3.0,
        //     //         z: 4.0
        //     //     }.into();
        //     // }

        //     // player.position.add_health.process(|pos| {
        //     //     println!("Pos {:?}", player.position.coords);
        //     //     // player.position.coords.x = 5.0;
        //     // })
        // }
    }
}

struct PlayerList {
    pub players: Vec<EntityId>,
}

fn main() {
    let worker_id = format!(
        "RustWorker{}",
        rand::thread_rng().gen::<u16>().to_string().as_str()
    );
    let conn = spatialos_gdk::worker::Connection::connect_with_receptionist(
        "RustWorker",
        "127.0.0.1",
        7777,
        worker_id.as_str(),
    );

    let mut world = World::<Schema>::new(conn);
    world.add_shared_resource(PlayerList{
        players: Vec::new()
    });
    let system = TestSystem {};
    world.register(Box::new(system));

    loop {
        world.process(0);
        thread::sleep(time::Duration::from_millis(250));
    }
}
