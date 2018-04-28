mod ffi;
mod dispatcher;
mod connection;
mod view;
mod entity;

pub use self::connection::Connection;
pub use self::dispatcher::Dispatcher;
pub use self::view::View;

type EntityId = i64;
type ComponentId = u32;

pub struct OpList {
	pointer: *const ffi::Worker_OpList
}