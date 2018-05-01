use libc::{c_char, c_void};
use worker::{EntityId, ComponentId};

#[repr(C)] pub struct Worker_ConnectionFuture { private: [u8; 0] }
#[repr(C)] pub struct Worker_ComponentVtable { pub private: [u8; 1000] }

#[repr(C)] pub struct Worker_RakNetParameters {
	heartbeat_timeout_millis: u32
}
#[repr(C)] pub struct Worker_TcpParameters {
	multiplex_level: u8,
	no_delay: u8,
	send_buffer_size: u32,
	receive_buffer_size: u32
}
#[repr(C)] pub struct Worker_NetworkParameters {
	flags: u8,
	connection_timeout_millis: u64,
	raknet: Worker_RakNetParameters,
	tcp: Worker_TcpParameters
}
#[repr(C)] pub struct Worker_ProtocolLoggingParameters {
	log_prefix: *const c_char,
	max_log_files: u32,
	max_log_file_size_bytes: u32
}
#[repr(C)] pub struct Worker_ConnectionParameters {
	pub worker_type: *const c_char,
	network: Worker_NetworkParameters,
	send_queue_capacity: u32,
	receive_queue_capacity: u32,
	log_message_queue_capacity: u32,
	built_in_metrics_report_period_millis: u32,
	protocol_logging: Worker_ProtocolLoggingParameters,
	enable_protocol_logging_at_startup: u8,
	component_vtable_count: u32,
	component_vtables: *const Worker_ComponentVtable,
	pub default_component_vtable: *const Worker_ComponentVtable
}


#[repr(C)] pub struct Worker_Connection { private: [u8; 0] }
#[repr(C)] pub struct Worker_Dispatcher { private: [u8; 0] }
#[repr(C)] pub struct Worker_OpList { private: [u8; 0] }

#[repr(C)] pub struct Worker_ComponentData {
	reserved: *const c_void,
	pub component_id: ComponentId,
	pub schema_type: *const c_void,
	user_handle: *const c_void
}
#[repr(C)] pub struct Worker_ComponentUpdate {
	reserved: *const c_void,
	pub component_id: ComponentId,
	pub schema_type: *const c_void,
	user_handle: *const c_void
}

#[repr(C)] pub struct Worker_CriticalSectionOp {
	pub in_critical_section: u8
}
#[repr(C)] pub struct Worker_AddEntityOp {
	pub entity_id: EntityId
}
#[repr(C)] pub struct Worker_AddComponentOp {
	pub entity_id: EntityId,
	pub data: Worker_ComponentData
}
#[repr(C)] pub struct Worker_ComponentUpdateOp {
	pub entity_id: EntityId,
	pub update: Worker_ComponentUpdate
}


#[allow(improper_ctypes)] 
#[link(name = "CWorker")]
extern {
	pub fn Worker_DefaultConnectionParameters() -> Worker_ConnectionParameters;

    pub fn Worker_ConnectAsync(hostname: *const c_char, 
    	port: u16, 
    	worker_id: *const c_char, 
    	params: *const Worker_ConnectionParameters) -> *const Worker_ConnectionFuture;

    pub fn Worker_ConnectionFuture_Get(future: *const Worker_ConnectionFuture, 
    	timeout_millis: Option<u32>) -> *const Worker_Connection;
    pub fn Worker_ConnectionFuture_Destroy(future: *const Worker_ConnectionFuture);

    pub fn Worker_Connection_Destroy(conn: *const Worker_Connection);
    pub fn Worker_Connection_GetOpList(conn: *const Worker_Connection, timeout_millis: u32) -> *const Worker_OpList;


    // Dispatcher
    pub fn Worker_Dispatcher_Create() -> *const Worker_Dispatcher;
    pub fn Worker_Dispatcher_Destroy(disp: *const Worker_Dispatcher);
    pub fn Worker_Dispatcher_Process(disp: *const Worker_Dispatcher, op_list: *const Worker_OpList);

    pub fn Worker_Dispatcher_SetCriticalSectionCallback(disp: *const Worker_Dispatcher,
    	data: *mut c_void, callback: extern fn(*mut c_void, *const Worker_CriticalSectionOp));
    pub fn Worker_Dispatcher_SetAddEntityCallback(disp: *const Worker_Dispatcher,
    	data: *mut c_void, callback: extern fn(*mut c_void, *const Worker_AddEntityOp));
    pub fn Worker_Dispatcher_SetAddComponentCallback(disp: *const Worker_Dispatcher,
    	data: *mut c_void, callback: extern fn(*mut c_void, *const Worker_AddComponentOp));
    pub fn Worker_Dispatcher_SetComponentUpdateCallback(disp: *const Worker_Dispatcher,
    	data: *mut c_void, callback: extern fn(*mut c_void, *const Worker_ComponentUpdateOp));
   
}