extern crate spatialos_gdk;

use schema::Schema;
use schema::improbable::{Position, Coordinates, Persistence};
use schema::demogame::Movement;
use spatialos_gdk::{EntityTemplate, Worker};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn main() {
    let filepath = "../snapshots/default.snapshot";

    let mut entities = vec![
        EntityTemplate::new(vec![Worker::Type("server")])
            .set_entity_id(1)
            .with_component(
                Worker::Type("server"),
                Position {
                    coords: Coordinates {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
            )
            .with_component(
                Worker::None,
                Persistence {},
            ),
        EntityTemplate::new(vec![Worker::Type("server")])
            .set_entity_id(2)
            .with_component(
                Worker::Type("server"),
                Position {
                    coords: Coordinates {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
            )
            .with_component(
                Worker::None,
                Persistence {},
            )
    ];

    for i in 0..1 {
        entities.push(EntityTemplate::new(vec![Worker::Type("server")])
            .set_entity_id(i + 5)
            .with_component(
                Worker::Type("server"),
                Position {
                    coords: Coordinates {
                        x: 0.0,
                        y: 0.0,
                        z: i as f64,
                    },
                },
            )
            .with_component(
                Worker::None,
                Persistence {},
            )
            .with_component(
                Worker::Type("server"),
                Movement {
                    moving_right: true
                },
            ));
    }

    ::spatialos_gdk::Snapshot::<Schema>::create(filepath, entities.into_iter());

    println!("Saved snapshot to {}", filepath);
}
