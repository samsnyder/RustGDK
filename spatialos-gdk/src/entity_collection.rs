use ComponentBitField;
use chunk::Chunk;
use component_group::ComponentGroup;
use component_group::UnsafeSendablePointer;
use entity::Entity;
use rayon::prelude::*;
use std::collections::HashMap;
use worker::Connection;
use worker::schema::GeneratedSchema;
use world::PartialEntity;
use world::WorldTime;

/// A view into the `World`'s entities, at a particular point in time.
///
/// This struct can be used to iterate over entities that match a particular
/// `ComponentGroup`.
///
/// This iteration can be done sequentially or in parallel.
pub struct Entities<'a, S: 'a + GeneratedSchema> {
    entities: &'a mut EntityCollection<S>,
    from_time: WorldTime,
}

impl<'a, S: 'static + GeneratedSchema> Entities<'a, S> {
    #[doc(hidden)]
    pub fn entities_from_time(
        entities: &'a mut EntityCollection<S>,
        from_time: &'a WorldTime,
    ) -> Entities<'a, S> {
        Entities {
            entities,
            from_time: from_time.clone(),
        }
    }

    /// Gets an iterator over all entities in the worker's local view which
    /// match the `ComponentGroup` `G`.
    pub fn get<'b, G: 'b + ComponentGroup<'b, S>>(&'b mut self) -> Box<Iterator<Item = G> + 'b> {
        let mut group_bit_field = S::ComponentBitField::new();
        G::add_to_bit_field(&mut group_bit_field);
        let from_time = &self.from_time;

        Box::new(
            self.entities
                .get_chunks_with_components(group_bit_field)
                .flat_map(move |chunk| G::get_iterator(chunk, from_time)),
        )
    }

    /// Executes the given closure for each entity in the worker's local view
    /// that matches the `ComponentGroup` `G`.
    ///
    /// The closure will likely be run in parallel, however if it is more effective
    /// to execute them in series, that will be done. Please see the [rayon](https://docs.rs/rayon/1.0.2/rayon)
    /// documentation for more details.
    pub fn par_for_each<'b, G: 'b + ComponentGroup<'b, S>, F: Send + Sync>(&'b mut self, cb: F)
    where
        F: Fn(&mut G),
    {
        let mut group_bit_field = S::ComponentBitField::new();
        G::add_to_bit_field(&mut group_bit_field);
        let from_time = &self.from_time;

        self.entities
            .par_for_each_chunks_with_components(group_bit_field, |chunk| {
                G::par_for_each(chunk, from_time, &cb);
            })
    }
}

pub struct EntityCollection<S: GeneratedSchema> {
    chunks: Vec<Chunk<S>>,
    map: HashMap<S::ComponentBitField, Vec<usize>>,
}

impl<S: 'static + GeneratedSchema> EntityCollection<S> {
    pub fn new() -> EntityCollection<S> {
        EntityCollection {
            chunks: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn replicate(&mut self, connection: &mut Connection) {
        for chunk in self.chunks.iter_mut() {
            chunk.replicate(connection);
        }
    }

    pub fn cleanup_after_frame(&mut self) {
        for chunk in self.chunks.iter_mut() {
            chunk.cleanup_after_frame();
        }
    }

    pub fn get_chunk_for_entity(&mut self, entity: &Entity<S>) -> &mut Chunk<S> {
        &mut self.chunks[entity.chunk_index]
    }

    pub fn get_free_chunk(
        &mut self,
        bit_field: &S::ComponentBitField,
        world_time: &mut WorldTime,
        template_entity: &PartialEntity<S>,
    ) -> &mut Chunk<S> {
        if let Some(indices) = self.map.get_mut(bit_field) {
            for index in indices {
                if (&self.chunks[*index]).has_space() {
                    return &mut self.chunks[*index];
                }
            }
        }

        // No space
        self.add_chunk(bit_field, world_time, template_entity)
    }

    pub fn get_chunks_with_components<'a>(
        &'a mut self,
        component_bit_field: S::ComponentBitField,
    ) -> Box<Iterator<Item = &mut Chunk<S>> + 'a> {
        let chunks: *mut Vec<Chunk<S>> = &mut self.chunks;

        Box::new(
            self.map
                .iter_mut()
                .filter(move |(bit_field, _)| bit_field.is_subset(&component_bit_field))
                .flat_map(move |(_, chunk_indices)| {
                    chunk_indices
                        .iter()
                        .map(move |index| unsafe { &mut (*chunks)[*index] })
                }),
        )
    }

    pub fn par_for_each_chunks_with_components<'b, F: Send + Sync>(
        &'b mut self,
        component_bit_field: S::ComponentBitField,
        cb: F,
    ) where
        F: Fn(&'b mut Chunk<S>),
    {
        let chunks =
            UnsafeSendablePointer::<Vec<Chunk<S>>>((&mut self.chunks) as *mut Vec<Chunk<S>>);

        self.map
            .par_iter_mut()
            .filter(|(bit_field, _)| bit_field.is_subset(&component_bit_field))
            .flat_map(|(_, chunk_indices)| {
                chunk_indices.par_iter().map(|index| unsafe {
                    let chunks_ref = &mut *(chunks.0);
                    let chunk = &mut chunks_ref[*index];
                    UnsafeSendablePointer::<Chunk<S>>(chunk)
                })
            })
            .for_each(|chunk_ptr| unsafe {
                let chunk = &mut (*chunk_ptr.0);
                cb(chunk);
            });
    }

    fn add_chunk(
        &mut self,
        bit_field: &S::ComponentBitField,
        world_time: &mut WorldTime,
        template_entity: &PartialEntity<S>,
    ) -> &mut Chunk<S> {
        let index = self.chunks.len();
        let chunk = Chunk::new(index, world_time, template_entity);
        self.chunks.push(chunk);
        self.map.entry(*bit_field).or_insert(Vec::new()).push(index);
        &mut self.chunks[index]
    }
}
