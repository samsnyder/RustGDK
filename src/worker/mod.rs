mod ffi;
pub mod dispatcher;
pub mod connection;

pub use self::connection::Connection;
pub use self::dispatcher::Dispatcher;

type EntityId = i64;
type ComponentId = u32;

pub struct OpList {
	pointer: *const ffi::Worker_OpList
}