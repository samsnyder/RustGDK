> DISCLAIMER: This is not an official Improbable GDK. I made this to try and learn Rust,
  and it is untested, undocumented and still missing features. Please use at your own risk!

> Also, I have only tested this on OS X, so it will likely not immediately work for Windows or Linux.

> Also, please give me Rust advice if you see something bad (style and best practices as well as bugs!)

# SpatialOS Rust GDK

This is a SpatialOS GDK built in Rust. It is an ECS which allows you to write systems which efficiently iterate over entities,
with any changes to components replicated over SpatialOS.

It supports:

* Fast iteration over [entities](https://docs.improbable.io/reference/latest/shared/glossary#entity),
  with a guaranteed linear memory layout
* Parallel iteration over entities
* Iterating over only [components](https://docs.improbable.io/reference/latest/shared/glossary#component) which have changed
* Sending and receiving [events](https://docs.improbable.io/reference/latest/shared/glossary#event)
* Sending and receiving [commands](https://docs.improbable.io/reference/latest/shared/glossary#command)
* Creating and deleting entities
* Shared local resources between systems
* All in Rust!

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
$ cd demo-project/workers/server
$ cargo doc -p spatialos-gdk --open
```

and look at the `spatialos-gdk` crate.

## Running the demo project

```shell
$ cd demo-project
$ spatial worker build -t=debug
$ spatial local launch
```

If all goes well, you should see an entity in the inspector move back and forth. Building the worker
for the first time will also take some time as `cargo` is building all of the dependencies.

## Repository structure

* `spatialos-gdk` contains the GDK crate itself.
	* `spatialos-gdk/spatialos-gdk-derive` contains the `ComponentGroup` custom derive macro.
	* `spatialos-gdk/spatialos-gdk-codegen` contains the code generator.
* `demo-project` contains a blank project which has one Rust worker type.
	* `demo-project/snapshot` contains a snapshot generation tool.
	* `demo-project/workers/server` contains the single managed worker.

