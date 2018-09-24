use TagComponentArray;
use downcast_rs::Downcast;
use entity::Entity;
use std::cell::RefCell;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use worker::schema::DynamicComponentHandler;
use worker::schema::{Component, GeneratedSchema};
use worker::schema::{ComponentDataInterface, ComponentUpdateInterface};
use worker::{Authority, ComponentId, Connection, EntityId};
use world::{PartialEntity, WorldTime};

pub const MAX_ENTITIES_PER_CHUNK: usize = 1024;

#[derive(Default)]
pub struct ComponentDataEntry<S: GeneratedSchema, C: Component<S>> {
    pub data: C::Data,
    pub last_updated: WorldTime,
}

#[doc(hidden)]
pub struct ComponentStorage<S: GeneratedSchema, C: Component<S>> {
    data: Vec<ComponentDataEntry<S, C>>,
    authority: TagComponentArray,
    last_updated: WorldTime,
    has_events_this_frame: bool,
    is_dirty: bool,
}

pub trait ComponentStorageInterface<S: GeneratedSchema>: Downcast {
    fn update_last_updated(&mut self, world_time: &mut WorldTime);
    fn set_component_data(
        &mut self,
        world_time: &mut WorldTime,
        entity_index: usize,
        data: S::ComponentData,
        authority: Authority,
    );

    fn apply_component_update(
        &mut self,
        world_time: &mut WorldTime,
        entity_index: usize,
        update: &S::ComponentUpdate,
    ) -> bool;
    fn set_authority(&mut self, entity_index: usize, authority: Authority);
    fn replicate(
        &mut self,
        entity_ids: &[EntityId],
        num_entities: usize,
        connection: &mut Connection,
    );
    fn cleanup_after_frame(&mut self, num_entities: usize);
    fn mark_as_dirty(&mut self);
    fn swap_entity(&mut self, from: usize, to: usize);
}

#[allow(dead_code)]
mod downcast {
    use chunk::ComponentStorageInterface;
    use worker::schema::GeneratedSchema;

    impl_downcast!(ComponentStorageInterface<S> where S: GeneratedSchema);
}

impl<S: GeneratedSchema, C: Component<S>> ComponentStorage<S, C> {
    fn new(world_time: &mut WorldTime) -> ComponentStorage<S, C> {
        let mut storage_vec: Vec<ComponentDataEntry<S, C>> =
            Vec::with_capacity(MAX_ENTITIES_PER_CHUNK);
        for _ in 0..MAX_ENTITIES_PER_CHUNK {
            storage_vec.push(Default::default());
        }

        ComponentStorage {
            data: storage_vec,
            authority: TagComponentArray::new(),
            last_updated: world_time.get_time(),
            has_events_this_frame: false,
            is_dirty: false,
        }
    }

    pub fn get_component_data_entry(
        &mut self,
        entity_index: usize,
    ) -> &mut ComponentDataEntry<S, C> {
        &mut self.data[entity_index]
    }

    pub fn get_authority(&mut self, entity_index: usize) -> Authority {
        if self.authority.get_tag(entity_index) {
            Authority::Authoritative
        } else {
            Authority::NotAuthoritative
        }
    }
}

impl<C: Component<S> + 'static, S: 'static + GeneratedSchema> ComponentStorageInterface<S>
    for ComponentStorage<S, C>
{
    fn update_last_updated(&mut self, world_time: &mut WorldTime) {
        self.last_updated = world_time.get_time();
    }

    fn set_component_data(
        &mut self,
        world_time: &mut WorldTime,
        entity_index: usize,
        data: S::ComponentData,
        authority: Authority,
    ) {
        let data_entry = ComponentDataEntry::<S, C> {
            data: C::extract_data(data).unwrap(),
            last_updated: world_time.get_time(),
        };
        self.data[entity_index] = data_entry;
        self.last_updated = world_time.get_time();
        self.set_authority(entity_index, authority);
    }

    // True if the update contains events
    fn apply_component_update(
        &mut self,
        world_time: &mut WorldTime,
        entity_index: usize,
        update: &S::ComponentUpdate,
    ) -> bool {
        let update = C::extract_update(update).unwrap();
        let contains_events = update.contains_events();
        if contains_events {
            self.has_events_this_frame = true;
        }

        let entry = &mut self.data[entity_index];
        C::apply_update_to_data(&mut entry.data, update);
        entry.last_updated = world_time.get_time();
        self.last_updated = world_time.get_time();

        contains_events
    }

    fn set_authority(&mut self, entity_index: usize, authority: Authority) {
        self.authority
            .set_tag(entity_index, authority != Authority::NotAuthoritative);
    }

    fn mark_as_dirty(&mut self) {
        self.is_dirty = true;
    }

    fn replicate(
        &mut self,
        entity_ids: &[EntityId],
        num_entities: usize,
        connection: &mut Connection,
    ) {
        if self.is_dirty {
            for index in 0..num_entities {
                let entry = self.get_component_data_entry(index);
                let entity_id = entity_ids[index];
                if entry.data.get_and_clear_dirty_bit() {
                    let update = entry.data.serialise_update();
                    connection.send_component_update(entity_id, C::component_id(), update);
                }
            }

            self.is_dirty = false;
        }
    }

    fn cleanup_after_frame(&mut self, num_entities: usize) {
        if self.has_events_this_frame {
            for index in 0..num_entities {
                let entry = self.get_component_data_entry(index);
                entry.data.cleanup_after_frame();
            }

            self.has_events_this_frame = false;
        }
    }

    fn swap_entity(&mut self, from: usize, to: usize) {
        self.data.swap(from, to);
        let from_authority = self.authority.get_tag(from);
        let to_authority = self.authority.get_tag(to);
        self.authority.set_tag(to, from_authority);
        self.authority.set_tag(from, to_authority);
    }
}

#[doc(hidden)]
pub struct Chunk<S: GeneratedSchema> {
    chunk_index: usize,
    entities: Vec<Rc<RefCell<Entity<S>>>>,
    entity_ids: [EntityId; MAX_ENTITIES_PER_CHUNK],
    data: HashMap<ComponentId, Box<ComponentStorageInterface<S>>>,
    num_entities: usize,
    is_dirty: bool,
    has_events_this_frame: bool,
    component_ids: HashSet<ComponentId>,
}

impl<S: 'static + GeneratedSchema> DynamicComponentHandler<S> for Chunk<S> {
    fn register_component<C: 'static + Component<S>>(&mut self) {
        if self.component_ids.contains(&C::component_id()) {
            let data_array: ComponentStorage<S, C> = ComponentStorage::new(&mut WorldTime::new());
            let data_array: Box<ComponentStorageInterface<S>> =
                Box::new(data_array) as Box<ComponentStorageInterface<S>>;

            self.data.insert(C::component_id(), data_array);
        }
    }
}

impl<S: 'static + GeneratedSchema> Chunk<S> {
    pub fn new(
        chunk_index: usize,
        world_time: &mut WorldTime,
        template_entity: &PartialEntity<S>,
    ) -> Chunk<S> {
        let component_ids = template_entity
            .component_data
            .keys()
            .map(|v| *v)
            .collect::<HashSet<ComponentId>>();

        let mut entities_vec: Vec<Rc<RefCell<Entity<S>>>> =
            Vec::with_capacity(MAX_ENTITIES_PER_CHUNK);
        for _ in 0..MAX_ENTITIES_PER_CHUNK {
            entities_vec.push(Default::default());
        }

        let mut chunk = Chunk {
            chunk_index,
            entities: entities_vec,
            entity_ids: [0; MAX_ENTITIES_PER_CHUNK],
            data: HashMap::new(),
            num_entities: 0,
            is_dirty: false,
            has_events_this_frame: false,
            component_ids,
        };

        S::run_dynamic_component_handler(&mut chunk);

        for data in chunk.data.values_mut() {
            data.update_last_updated(world_time);
        }

        chunk
    }

    pub fn mark_component_storage_as_dirty<C: 'static + Component<S>>(&mut self) {
        self.is_dirty = true;
        self.get_component_storage_interface(C::component_id())
            .mark_as_dirty();
    }

    pub fn replicate(&mut self, connection: &mut Connection) {
        if self.is_dirty {
            for (_, mut storage) in &mut self.data {
                storage.replicate(&self.entity_ids, self.num_entities, connection);
            }

            self.is_dirty = false;
        }
    }

    pub fn cleanup_after_frame(&mut self) {
        if self.has_events_this_frame {
            for (_, mut storage) in &mut self.data {
                storage.cleanup_after_frame(self.num_entities);
            }

            self.has_events_this_frame = false;
        }
    }

    pub fn has_space(&self) -> bool {
        self.num_entities < MAX_ENTITIES_PER_CHUNK
    }

    pub fn entity_index_iter<'a>(&'a self) -> Box<Iterator<Item = usize> + 'a> {
        Box::new(0..self.num_entities)
    }

    pub fn par_for_each_entity_index<F: Send + Sync>(&self, op: F)
    where
        F: Fn(usize),
    {
        (0..self.num_entities).into_par_iter().for_each(op);
    }

    pub fn get_entity_id(&self, index: usize) -> EntityId {
        self.entity_ids[index]
    }

    pub fn add_entity(
        &mut self,
        world_time: &mut WorldTime,
        mut entity: PartialEntity<S>,
    ) -> Rc<RefCell<Entity<S>>> {
        let entity_index = self.num_entities;
        self.num_entities = self.num_entities + 1;

        let new_entity = Rc::new(RefCell::new(Entity::new(
            self.chunk_index,
            entity_index,
            entity.bit_field,
        )));

        self.entities[entity_index] = new_entity.clone();
        self.entity_ids[entity_index] = entity.entity_id;

        for (component_id, component_data) in entity.component_data.drain() {
            let authority = entity
                .write_authority
                .get(&component_id)
                .unwrap_or(&Authority::NotAuthoritative);
            self.get_component_storage_interface(component_id)
                .set_component_data(world_time, entity_index, component_data, *authority);
        }

        new_entity
    }

    pub fn remove_entity(&mut self, entity: &Entity<S>) {
        // Swap last entity with the new gap
        let entity_index = entity.index_in_chunk;
        let num_entities = self.num_entities;

        self.move_entity(num_entities - 1, entity_index);

        self.num_entities = self.num_entities - 1;
    }

    fn move_entity(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }

        self.entities.swap(from, to);
        self.entity_ids.swap(from, to);

        self.entities[to].borrow_mut().index_in_chunk = to;

        for (_, mut storage) in &mut self.data {
            storage.swap_entity(from, to);
        }
    }

    fn get_component_storage_interface(
        &mut self,
        component_id: ComponentId,
    ) -> &mut Box<ComponentStorageInterface<S>> {
        self.data.get_mut(&component_id).unwrap()
    }

    pub fn get_component_storage<C: 'static + Component<S>>(
        &mut self,
    ) -> Option<&mut ComponentStorage<S, C>> {
        self.get_component_storage_interface(C::component_id())
            .downcast_mut::<ComponentStorage<S, C>>()
    }

    pub fn apply_component_update(
        &mut self,
        component_id: ComponentId,
        world_time: &mut WorldTime,
        entity: &Entity<S>,
        update: S::ComponentUpdate,
    ) {
        let contains_events = self.get_component_storage_interface(component_id)
            .apply_component_update(world_time, entity.index_in_chunk, &update);
        if contains_events {
            self.has_events_this_frame = true;
        }
    }

    pub fn apply_authority(
        &mut self,
        component_id: ComponentId,
        entity: &Entity<S>,
        authority: Authority,
    ) {
        self.get_component_storage_interface(component_id)
            .set_authority(entity.index_in_chunk, authority);
    }
}
