extern crate boxfnonce;
extern crate libc;
extern crate rayon;
#[macro_use]
extern crate downcast_rs;

mod chunk;
mod commands;
mod component_group;
mod entity;
mod entity_collection;
mod entity_template;
mod shared_resources;
mod snapshot;
mod system;
mod world;

#[doc(hidden)]
pub mod worker;

pub use self::chunk::{Chunk, ComponentStorage};
pub use self::component_group::{ComponentGroup, ModifiedRead, ModifiedWrite, Read,
                                UnsafeSendablePointer, Write};
pub use self::entity_collection::Entities;
pub use self::entity_template::{EntityTemplate, Worker};
pub use self::snapshot::Snapshot;
pub use self::system::System;
pub use self::worker::{Connection, ConnectionParameters, EntityId, LogLevel};
pub use self::world::{World, WorldError, WorldTime};

use chunk::MAX_ENTITIES_PER_CHUNK;
use worker::ComponentId;

#[doc(hidden)]
pub const FIELD_SIZE_BITS: usize = 64;

#[doc(hidden)]
pub trait ComponentBitField: Default {
    const NUMBER_OF_FIELDS: usize;

    fn new() -> Self;

    fn get_field(&self, field_index: usize) -> &u64;
    fn get_field_mut(&mut self, field_index: usize) -> &mut u64;

    fn get_unique_index(component_id: ComponentId) -> Option<usize>;

    fn add_component(&mut self, component_id: ComponentId) -> bool {
        if let Some(unique_index) = Self::get_unique_index(component_id) {
            let field_index = unique_index / FIELD_SIZE_BITS;
            let field_offset = unique_index % FIELD_SIZE_BITS;

            let field = self.get_field_mut(field_index);
            *field |= 1 << field_offset;

            true
        } else {
            false
        }
    }

    fn is_subset(&self, subset: &Self) -> bool {
        for i in 0..Self::NUMBER_OF_FIELDS {
            if (!(*self.get_field(i))) & (*(subset.get_field(i))) != 0 {
                return false;
            }
        }
        true
    }
}

const NUMBER_OF_TAG_FIELDS: usize = 1 + (MAX_ENTITIES_PER_CHUNK / 64);

struct TagComponentArray {
    pub fields: [u64; NUMBER_OF_TAG_FIELDS],
}

impl TagComponentArray {
    fn new() -> TagComponentArray {
        TagComponentArray {
            fields: [0; NUMBER_OF_TAG_FIELDS],
        }
    }

    fn set_tag(&mut self, index: usize, has_tag: bool) {
        let field_index = index / 64;
        let field_offset = index % 64;

        let field = &mut self.fields[field_index];
        if has_tag {
            *field |= 1 << field_offset;
        } else {
            *field &= !(1 << field_offset);
        }
    }

    fn get_tag(&self, index: usize) -> bool {
        let field_index = index / 64;
        let field_offset = index % 64;

        let field = &self.fields[field_index];
        let value = (*field >> field_offset) & 1;
        value == 1
    }

    // fn iter<'a>(&'a self) -> Box<Iterator<Item = usize> + 'a> {
    //     Box::new((0..MAX_ENTITIES_PER_CHUNK).filter(move |index| self.get_tag(*index)))
    // }

    // fn par_for_each<F: Send + Sync>(&self, op: F)
    // where
    //     F: Fn(usize),
    // {
    //     (0..MAX_ENTITIES_PER_CHUNK)
    //         .into_par_iter()
    //         .filter(move |index| self.get_tag(*index))
    //         .for_each(op);
    // }
}
