use worker::ffi::{Schema_CommandRequest, Schema_CommandResponse, Schema_ComponentData,
                       Schema_ComponentUpdate};

use ComponentBitField;
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use worker::ComponentId;

pub type FieldId = u32;

pub trait GeneratedSchema: Sized + Default {
    const NUMBER_OF_COMPONENTS: usize;
    type ComponentData: GlobalComponentDataInterface<Self>;
    type ComponentUpdate: GlobalComponentUpdateInterface<Self>;
    type ComponentBitField: ComponentBitField
        + Clone
        + Copy
        + PartialEq
        + PartialOrd
        + Eq
        + Hash
        + Debug
        + Default
        + Send
        + Sync;

    fn serialise_entity_acl(
        read: Vec<String>,
        write: HashMap<ComponentId, String>,
    ) -> (ComponentId, Box<Schema_ComponentData>);
    fn run_dynamic_component_handler<D: DynamicComponentHandler<Self>>(handler: &mut D);
    unsafe fn deserialise_command_request(
        component_id: ComponentId,
        command_index: u32,
        request: Box<Schema_CommandRequest>,
    ) -> Option<Box<Any>>;
    unsafe fn deserialise_command_response(
        component_id: ComponentId,
        command_index: u32,
        response: Box<Schema_CommandResponse>,
    ) -> Option<Box<Any>>;
}

pub trait GlobalComponentDataInterface<S: GeneratedSchema> {
    fn deserialise(
        component_id: ComponentId,
        data: Box<Schema_ComponentData>,
    ) -> Option<S::ComponentData>;
    fn serialise(&self) -> Box<Schema_ComponentData>;
    fn apply_update(&mut self, update: &S::ComponentUpdate);
}

pub trait GlobalComponentUpdateInterface<S: GeneratedSchema> {
    fn deserialise(
        component_id: ComponentId,
        update: Box<Schema_ComponentUpdate>,
    ) -> Option<S::ComponentUpdate>;
}

pub trait Component<S: GeneratedSchema>: Default {
    type Data: ComponentDataInterface<S> + Default;
    type Update: ComponentUpdateInterface<S>;

    fn component_id() -> ComponentId;
    fn apply_update_to_data(data: &mut Self::Data, update: &Self::Update);
    fn extract_data_borrow(data: &S::ComponentData) -> Option<&Self::Data>;
    fn extract_data(data: S::ComponentData) -> Option<Self::Data>;
    fn extract_update(update: &S::ComponentUpdate) -> Option<&Self::Update>;
    fn serialise_snapshot(self) -> Box<Schema_ComponentData>;
}

pub trait ComponentDataInterface<S: GeneratedSchema>: Sized {
    fn deserialise_data(update: Box<Schema_ComponentData>) -> S::ComponentData;
    fn serialise_data(&self) -> Box<Schema_ComponentData>;
    // fn mark_as_dirty(&mut self, field_index: usize);
    fn serialise_update(&mut self) -> Box<Schema_ComponentUpdate>;
    fn get_and_clear_dirty_bit(&mut self) -> bool;
    // fn is_dirty(&self) -> bool;
    fn make_dirty(&mut self);
    fn cleanup_after_frame(&mut self);
}

pub trait ComponentUpdateInterface<S: GeneratedSchema>: Sized {
    fn deserialise_update(update: Box<Schema_ComponentUpdate>) -> S::ComponentUpdate;
    fn contains_events(&self) -> bool;
}

pub trait Command<S: GeneratedSchema> {
    type Component: Component<S>;
    type Request: CommandRequestInterface;
    type Response: CommandResponseInterface;

    fn command_index() -> u32;
}

pub trait CommandRequestInterface {
    fn deserialise_request(request: Box<Schema_CommandRequest>) -> Self;
    fn serialise_request(&self) -> Box<Schema_CommandRequest>;
}

pub trait CommandResponseInterface {
    fn deserialise_response(response: Box<Schema_CommandResponse>) -> Self;
    fn serialise_response(&self) -> Box<Schema_CommandResponse>;
}

pub trait DynamicComponentHandler<S: GeneratedSchema> {
    fn register_component<C: 'static + Component<S>>(&mut self);
}

#[derive(Clone, Debug, Default)]
pub struct Event<T> {
    events: Vec<T>,
    staged_events: Vec<T>,
}

impl<T> Event<T> {
    pub fn new() -> Event<T> {
        Event {
            events: Vec::new(),
            staged_events: Vec::new(),
        }
    }

    pub fn process<F>(&self, mut cb: F)
    where
        F: FnMut(&T),
    {
        for event in &self.events {
            cb(event);
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn trigger(&mut self, value: T) {
        self.staged_events.push(value);
    }

    pub fn add_event(&mut self, value: T) {
        self.events.push(value);
    }

    pub fn get_staged_events(&self) -> &Vec<T> {
        &self.staged_events
    }

    pub fn clear_staged_events(&mut self) {
        self.staged_events.clear()
    }
}

impl<T> IntoIterator for Event<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Event<T> {
    type Item = &'a T;
    type IntoIter = ::std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

#[derive(Clone, Default)]
pub struct Property<T: fmt::Debug> {
    is_dirty: bool,
    value: T,
}

impl<T: fmt::Debug> From<T> for Property<T> {
    fn from(value: T) -> Property<T> {
        Property {
            is_dirty: true,
            value,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Property<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: fmt::Debug> Property<T> {
    pub fn new(value: T) -> Property<T> {
        Property {
            is_dirty: false,
            value,
        }
    }

    pub fn get_dirty_bit(&self) -> bool {
        self.is_dirty
    }

    pub fn get_and_clear_dirty_bit(&mut self) -> bool {
        let dirty = self.is_dirty;
        self.is_dirty = false;
        dirty
    }
}

impl<T: fmt::Debug> Deref for Property<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: fmt::Debug> DerefMut for Property<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.is_dirty = true;
        &mut self.value
    }
}
