use chunk::Chunk;
use std::marker::Sized;
use std::ops::{Deref, DerefMut};
use worker::schema::ComponentDataInterface;
use worker::schema::{Component, GeneratedSchema};
use world::WorldTime;

#[doc(hidden)]
pub struct UnsafeSendablePointer<T>(pub *mut T);

unsafe impl<T> Sync for UnsafeSendablePointer<T> {}
unsafe impl<T> Send for UnsafeSendablePointer<T> {}

/// An immutable reference to component data.
///
/// This can be used in a `ComponentGroup` to match only entities
/// that contain the given component, and to retrieve but not edit
/// the component data.
pub struct Read<'a, S: 'static + GeneratedSchema, C: 'static + Component<S>> {
    data: &'a <C as Component<S>>::Data,
}
impl<'a, S: GeneratedSchema, C: 'static + Component<S>> Read<'a, S, C> {
    pub fn new(data: &'a <C as Component<S>>::Data) -> Read<'a, S, C> {
        Read { data }
    }
}
impl<'a, S: GeneratedSchema, C: 'static + Component<S>> Deref for Read<'a, S, C> {
    type Target = <C as Component<S>>::Data;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// A mutable reference to component data.
///
/// This can be used in a `ComponentGroup` to match only entities
/// that contain the given component as well as authority over the given component,
/// and to retrieve and be able to edit the component data.
pub struct Write<'a, S: 'static + GeneratedSchema, C: 'static + Component<S>> {
    data: &'a mut <C as Component<S>>::Data,
}
impl<'a, S: GeneratedSchema, C: 'static + Component<S>> Write<'a, S, C> {
    pub fn new(data: &'a mut <C as Component<S>>::Data) -> Write<'a, S, C> {
        Write { data }
    }
}
impl<'a, S: GeneratedSchema, C: 'static + Component<S>> Deref for Write<'a, S, C> {
    type Target = <C as Component<S>>::Data;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<'a, S: GeneratedSchema, C: 'static + Component<S>> DerefMut for Write<'a, S, C> {
    fn deref_mut(&mut self) -> &mut <C as Component<S>>::Data {
        self.data.make_dirty();
        &mut self.data
    }
}

/// Like `Read`, however this will only match entities where the component data
/// has changed since the last frame.
pub type ModifiedRead<'a, S, C> = Read<'a, S, C>;

/// Like `Write`, however this will only match entities where the component data
/// has changed since the last frame.
pub type ModifiedWrite<'a, S, C> = Write<'a, S, C>;

#[doc(hidden)]
pub trait ComponentGroup<'a, S: GeneratedSchema>
where
    Self: Sized,
{
    fn add_to_bit_field(bit_field: &mut S::ComponentBitField);
    fn get_iterator(
        chunk: &'a mut Chunk<S>,
        from_time: &'a WorldTime,
    ) -> Box<Iterator<Item = Self> + 'a>;
    fn par_for_each<F: Send + Sync>(chunk: &'a mut ::Chunk<S>, from_time: &'a WorldTime, cb: F)
    where
        F: Fn(&mut Self);
}
