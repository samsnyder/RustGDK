extern crate rand;
extern crate spatialos_gdk;
#[macro_use]
extern crate spatialos_gdk_derive;

use rand::Rng;
use schema::Schema;
use schema::demogame::Movement;
use schema::improbable::Position;
use spatialos_gdk::{Connection, ConnectionParameters, Entities, EntityId, System,
                    World, WorldError, Write};
use std::{env, thread, time};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

const FRAME_INTERVAL_S: f64 = 1.0 / 30.0;

#[derive(ComponentGroup)]
pub struct MovementData<'a> {
    pub movement: Write<'a, Schema, Movement>,
    pub position: Write<'a, Schema, Position>,
    pub entity_id: EntityId,
}

struct MovementSystem {}

impl System<Schema> for MovementSystem {
    fn on_update(&mut self, _world: &mut World<Schema>, entities: &mut Entities<Schema>) {
        for mut entity in entities.get::<MovementData>() {
            if *entity.movement.moving_right && entity.position.coords.x > 10.0 {
                *entity.movement.moving_right = false;
            } else if !*entity.movement.moving_right && entity.position.coords.x < -10.0 {
                *entity.movement.moving_right = true;
            }

            let delta_x = FRAME_INTERVAL_S * 4.0 * (if *entity.movement.moving_right {
                1.0
            } else {
                -1.0
            });
            entity.position.coords.x += delta_x;
        }
    }
}

fn get_connection(params: ConnectionParameters) -> Connection {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.len() == 0 {
        let worker_id = format!(
            "server{}",
            rand::thread_rng().gen::<u16>().to_string().as_str()
        );
        Connection::connect_with_receptionist(
            "server",
            "127.0.0.1",
            7777,
            worker_id.as_str(),
            params,
        )
    } else if args.len() == 4 {
        if args[0] == "receptionist" {
            Connection::connect_with_receptionist(
                "server",
                args[1].as_str(),
                args[2].parse().unwrap(),
                args[3].as_str(),
                params,
            )
        } else {
            panic!("Unknown connection type: {}", args[0]);
        }
    } else {
        panic!("Must have 0 arguments for default connection or 4 arguments for configured connection\n
			    of the form <receptionist> <receptionist_ip> <receptionist_port> <worker_id>");
    }
}

fn main() {
    let mut world = World::<Schema>::new(get_connection(ConnectionParameters::default()));

    world.register(MovementSystem {});

    loop {
        if let Result::Err(WorldError::ConnectionLost) = world.process(0) {
            panic!("Connection lost.")
        }
        thread::sleep(time::Duration::from_millis(
            (FRAME_INTERVAL_S * 1000.0) as u64,
        ));
    }
}
