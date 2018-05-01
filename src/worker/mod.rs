mod ffi;
mod dispatcher;
mod connection;
mod view;
mod entity;
mod component;

pub use self::connection::Connection;
pub use self::dispatcher::Dispatcher;
pub use self::view::View;
pub use self::component::{ComponentMetaclass, ComponentUpdate};

pub type EntityId = i64;
pub type ComponentId = u32;

pub struct OpList {
	pointer: *const ffi::Worker_OpList
}