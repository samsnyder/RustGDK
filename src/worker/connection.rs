use worker::{ffi, OpList};
use std::ffi::CString;

const DEFAULT_VTABLE: ffi::Worker_ComponentVtable = ffi::Worker_ComponentVtable { private: [0; 1000] };

pub struct Connection {
	pointer: *const ffi::Worker_Connection
}

impl Drop for Connection {
	fn drop(&mut self) {
		unsafe {
			ffi::Worker_Connection_Destroy(self.pointer);
		}
	}
}

impl Connection {
	pub fn connect_with_receptionist(worker_type: &str, hostname: &str, port: u16, worker_id: &str) -> Connection {
		unsafe {
			let worker_type = CString::new(worker_type).unwrap();
			let hostname = CString::new(hostname).unwrap();
			let worker_id = CString::new(worker_id).unwrap();

			let mut params = ffi::Worker_DefaultConnectionParameters();
			params.worker_type = worker_type.as_ptr();
			params.default_component_vtable = &DEFAULT_VTABLE;

			let future = ffi::Worker_ConnectAsync(hostname.as_ptr(),
				port,
				worker_id.as_ptr(),
				&params);
			let pointer = ffi::Worker_ConnectionFuture_Get(future, Some(1000));
			Connection {
				pointer
			}
		}
	}

	pub fn get_op_list(&self, timeout_millis: u32) -> OpList {
		unsafe {
			let pointer = ffi::Worker_Connection_GetOpList(self.pointer, timeout_millis);
			OpList {
				pointer
			}
		}
	}
}