> This is not an official Improbable GDK. I made this to try and learn Rust,
  and it is **untested**, **undocumented** and still **missing features**. Please use at your own risk!

> Also, I have only tested this on OS X, so it will likely not immediately work for Windows or Linux.

> Also, please give me Rust advice if you see something bad (style and best practices as well as bugs!)

# SpatialOS Rust GDK

This is a SpatialOS GDK built in Rust. It is an ECS which allows you to write systems which efficiently iterate over entities,
with any changes to components replicated over SpatialOS.

It supports:

* Iteration over [entities](https://docs.improbable.io/reference/latest/shared/glossary#entity),
  with a guaranteed linear memory layout (implemented in a similar way to the Unity ECS)
* Parallel iteration over entities
* Iterating over only [components](https://docs.improbable.io/reference/latest/shared/glossary#component) which have changed
* Sending and receiving [events](https://docs.improbable.io/reference/latest/shared/glossary#event)
* Sending and receiving [commands](https://docs.improbable.io/reference/latest/shared/glossary#command)
* Creating and deleting entities
* Shared local resources between systems
* All in Rust!

It does not support (but I plan to add):

* Non-SpatialOS components
* Schema enums
* The Locator
* Entity queries
* Worker flags
* Reading snapshots
* Probably a load of other C SDK features...

## Example

```rust
#[derive(ComponentGroup)]
pub struct MovementData<'a> {
    pub position: Write<'a, Schema, Position>,
    pub metadata: Read<'a, Schema, Metadata>
}

struct MovementSystem {}

impl System<Schema> for MovementSystem {
    fn on_update(&mut self, _world: &mut World<Schema>, entities: &mut Entities<Schema>) {
        for mut entity in entities.get::<MovementData>() {
            entity.position.coords.x = rand::thread_rng().gen::<f64>();

            println!("Entity of type {} has an x value of {}", 
            	*entity.metadata.entity_type, 
            	entity.position.coords.x);
        }
    }
}
```

A full example can be seen in the [demo project](demo-project/workers/server/src/main.rs).

Please see the documentation for more examples of how to use the API. To open the documentation,
please run:

```shell
$ cd spatialos-gdk
$ cargo doc -p spatialos-gdk --open
```

## Running the demo project

```shell
$ cd demo-project
$ spatial worker build -t=debug
$ spatial local launch
```

Building the worker for the first time will take a while because `cargo` is building all of the dependencies.

If all goes well, you should see an entity in the inspector move back and forth.

### Generating the snapshot

```shell
$ cd demo-project/snapshot
$ cargo run
```

## The build process

Inside the worker's [`build.rs`](demo-project/workers/server/src/build.rs) file, the code generation is run.
This means that there is no need to run `spatial codegen` if you are just using Rust workers.

If you make any changes to your worker or schema, you can simply run `cargo run` and it will generate the
code and run the worker with the default parameters (connecting to the receptionist on `localhost`).

If you are developing a managed worker, you will need to re-run `spatial build -t=debug` instead, as this runs `cargo build`
but also zips the artifact up for SpatialOS to use.

## Repository structure

* `spatialos-gdk` contains the GDK crate itself.
	* `spatialos-gdk/spatialos-gdk-derive` contains the `ComponentGroup` custom derive macro.
	* `spatialos-gdk/spatialos-gdk-codegen` contains the code generator.
* `demo-project` contains a blank project which has one Rust worker type.
	* `demo-project/snapshot` contains a snapshot generation tool.
	* `demo-project/workers/server` contains the single managed worker.

## Known issues

* Serialising a byte array or a String currently leaks the memory.
