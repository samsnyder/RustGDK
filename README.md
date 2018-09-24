> DISCLAIMER: This is not an official Improbable SDK. I made this to try and learn Rust,
  and it is untested, undocumented and still missing features. Please use at your own risk!

# SpatialOS Rust SDK

This is a SpatialOS SDK built in Rust. It is an ECS which allows you to write systems which efficiently iterate over entities,
with any changes to components replicated over SpatialOS.

It supports:

* Fast iteration over entities, with a guaranteed linear memory layout.
* Parallel iteration over entities.
* Iterating over only components which have changed.
* Sending and receiving events.
* Sending and receiving commands.
* Creating and deleting entities.
* Shared local resources between systems.
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

### Running the demo project

```
$ cd demo-project
$ spatial worker build -t=debug
$ spatial local launch
```

You should see an entity in the inspector move back and forth.

