use entity_collection::Entities;
use worker::schema::GeneratedSchema;
use world::World;

/// A trait indicating that this struct will act as a system in this worker.
#[allow(unused_variables)]
pub trait System<S: GeneratedSchema> {
    /// This is called when the system is registered to the `World`, and can be optionally
    /// overriden to perform initial tasks such as registering command handlers.
    fn on_ready(&mut self, world: &mut World<S>) {}

    /// This is called in every `World` tick. It may perform operations in the `World` as
    /// well as iterate over entities that match a given `ComponentGroup`.
    fn on_update(&mut self, world: &mut World<S>, entities: &mut Entities<S>) {}
}
