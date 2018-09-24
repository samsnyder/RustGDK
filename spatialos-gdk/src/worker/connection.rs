use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;
use worker::{ffi, ComponentId, EntityId, FFIEnum, LogLevel, OpList, RequestId};

pub enum ConnectionType {
    RakNet,
    TCP,
}

pub struct NetworkParameters {
    use_external_ip: bool,
    connection_type: ConnectionType,
    raknet_heartbeat_timeout_millis: u32,
    tcp_multiplex_level: u8,
    tcp_send_buffer_size: u32,
    tcp_receive_buffer_size: u32,
    tcp_no_delay: bool,
    connection_timeout_millis: u64,
}

impl Default for NetworkParameters {
    fn default() -> NetworkParameters {
        unsafe {
            let ffi_params = ffi::Worker_DefaultConnectionParameters().network;
            NetworkParameters {
                use_external_ip: ffi_params.use_external_ip != 0,
                connection_type: match ffi_params.connection_type as u32 {
                    ffi::Worker_NetworkConnectionType::WORKER_NETWORK_CONNECTION_TYPE_TCP => {
                        ConnectionType::TCP
                    }
                    ffi::Worker_NetworkConnectionType::WORKER_NETWORK_CONNECTION_TYPE_RAKNET => {
                        ConnectionType::RakNet
                    }
                    unkown => panic!("Unknown network protocol: {}", unkown),
                },
                raknet_heartbeat_timeout_millis: ffi_params.raknet.heartbeat_timeout_millis,
                tcp_multiplex_level: ffi_params.tcp.multiplex_level,
                tcp_send_buffer_size: ffi_params.tcp.send_buffer_size,
                tcp_receive_buffer_size: ffi_params.tcp.receive_buffer_size,
                tcp_no_delay: ffi_params.tcp.no_delay != 0,
                connection_timeout_millis: ffi_params.connection_timeout_millis,
            }
        }
    }
}

impl From<NetworkParameters> for ffi::Worker_NetworkParameters {
    fn from(value: NetworkParameters) -> Self {
        unsafe {
            let mut ffi_params = ffi::Worker_DefaultConnectionParameters().network;
            ffi_params.use_external_ip = if value.use_external_ip { 1 } else { 0 };
            ffi_params.connection_type = match value.connection_type {
                ConnectionType::TCP => {
                    ffi::Worker_NetworkConnectionType::WORKER_NETWORK_CONNECTION_TYPE_TCP
                }
                ConnectionType::RakNet => {
                    ffi::Worker_NetworkConnectionType::WORKER_NETWORK_CONNECTION_TYPE_RAKNET
                }
            } as u8;
            ffi_params.raknet.heartbeat_timeout_millis = value.raknet_heartbeat_timeout_millis;
            ffi_params.tcp.multiplex_level = value.tcp_multiplex_level;
            ffi_params.tcp.send_buffer_size = value.tcp_send_buffer_size;
            ffi_params.tcp.receive_buffer_size = value.tcp_receive_buffer_size;
            ffi_params.tcp.no_delay = if value.tcp_no_delay { 1 } else { 0 };
            ffi_params.connection_timeout_millis = value.connection_timeout_millis;

            ffi_params
        }
    }
}

pub struct ConnectionParameters {
    pub network: NetworkParameters,
    pub send_queue_capacity: u32,
    pub receive_queue_capacity: u32,
    pub log_message_queue_capacity: u32,
    pub built_in_metrics_report_period_millis: u32,
    pub protocol_log_prefix: String,
    pub max_protocol_log_files: u32,
    pub max_protocol_log_file_size_bytes: u32,
    pub enable_protocol_logging_at_startup: bool,
}

impl Default for ConnectionParameters {
    fn default() -> ConnectionParameters {
        unsafe {
            let ffi_params = ffi::Worker_DefaultConnectionParameters();
            ConnectionParameters {
                network: NetworkParameters::default(),
                send_queue_capacity: ffi_params.send_queue_capacity,
                receive_queue_capacity: ffi_params.receive_queue_capacity,
                log_message_queue_capacity: ffi_params.log_message_queue_capacity,
                built_in_metrics_report_period_millis: ffi_params
                    .built_in_metrics_report_period_millis,
                enable_protocol_logging_at_startup: ffi_params.enable_protocol_logging_at_startup
                    != 0,
                protocol_log_prefix: CStr::from_ptr(ffi_params.protocol_logging.log_prefix)
                    .to_owned()
                    .into_string()
                    .unwrap(),
                max_protocol_log_files: ffi_params.protocol_logging.max_log_files,
                max_protocol_log_file_size_bytes: ffi_params
                    .protocol_logging
                    .max_log_file_size_bytes,
            }
        }
    }
}

impl From<ConnectionParameters> for ffi::Worker_ConnectionParameters {
    fn from(value: ConnectionParameters) -> Self {
        unsafe {
            let mut ffi_params = ffi::Worker_DefaultConnectionParameters();
            ffi_params.network = ffi::Worker_NetworkParameters::from(value.network);
            ffi_params.send_queue_capacity = value.send_queue_capacity;
            ffi_params.receive_queue_capacity = value.receive_queue_capacity;
            ffi_params.log_message_queue_capacity = value.log_message_queue_capacity;
            ffi_params.built_in_metrics_report_period_millis =
                value.built_in_metrics_report_period_millis;
            ffi_params.enable_protocol_logging_at_startup =
                if value.enable_protocol_logging_at_startup {
                    1
                } else {
                    0
                };
            // This will leak this string.
            ffi_params.protocol_logging.log_prefix =
                CString::new(value.protocol_log_prefix).unwrap().into_raw();
            ffi_params.protocol_logging.max_log_files = value.max_protocol_log_files;
            ffi_params.protocol_logging.max_log_file_size_bytes =
                value.max_protocol_log_file_size_bytes;

            ffi_params
        }
    }
}

pub struct Connection {
    pointer: *mut ffi::Worker_Connection,
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            ffi::Worker_Connection_Destroy(self.pointer);
        }
    }
}

impl Connection {
    pub fn default_vtable() -> Box<ffi::Worker_ComponentVtable> {
        unsafe { Box::new(mem::zeroed()) }
    }

    pub fn connect_with_receptionist(
        worker_type: &str,
        hostname: &str,
        port: u16,
        worker_id: &str,
        params: ConnectionParameters,
    ) -> Connection {
        unsafe {
            let worker_type = CString::new(worker_type).unwrap();
            let hostname = CString::new(hostname).unwrap();
            let worker_id = CString::new(worker_id).unwrap();

            let default_vtable_ptr = Box::leak(Connection::default_vtable());

            let mut params = Box::new(ffi::Worker_ConnectionParameters::from(params));
            params.worker_type = worker_type.into_raw();
            params.default_component_vtable = default_vtable_ptr;

            let params = Box::leak(params);
            let future =
                ffi::Worker_ConnectAsync(hostname.into_raw(), port, worker_id.into_raw(), params);
            let pointer = ffi::Worker_ConnectionFuture_Get(future, ptr::null());
            ffi::Worker_ConnectionFuture_Destroy(future);

            Connection { pointer }
        }
    }

    pub fn get_op_list(&mut self, timeout_millis: u32) -> OpList {
        unsafe {
            let pointer = ffi::Worker_Connection_GetOpList(self.pointer, timeout_millis);
            OpList::new(pointer)
        }
    }

    pub fn is_connected(&self) -> bool {
        unsafe { ffi::Worker_Connection_IsConnected(self.pointer) != 0 }
    }

    pub fn send_log_message(&mut self, level: LogLevel, logger_name: String, message: String) {
        unsafe {
            let logger_name = CString::new(logger_name).unwrap();
            let message = CString::new(message).unwrap();
            let log_message = ffi::Worker_LogMessage {
                level: level.get_u8(),
                logger_name: logger_name.as_ptr(),
                message: message.as_ptr(),
                entity_id: ptr::null(),
            };
            ffi::Worker_Connection_SendLogMessage(
                self.pointer,
                &log_message as *const ffi::Worker_LogMessage,
            );
        }
    }

    pub fn send_component_update(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        update: Box<ffi::Schema_ComponentUpdate>,
    ) {
        unsafe {
            let mut component_update: ffi::Worker_ComponentUpdate = mem::zeroed();
            component_update.component_id = component_id;
            component_update.schema_type = Box::into_raw(update);

            ffi::Worker_Connection_SendComponentUpdate(self.pointer, entity_id, &component_update);
            Box::from_raw(component_update.schema_type);
        }
    }

    pub fn send_command_request(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        request: Box<ffi::Schema_CommandRequest>,
        command_id: u32,
        timeout_millis: Option<u32>,
    ) -> RequestId {
        unsafe {
            let mut command_request: ffi::Worker_CommandRequest = mem::zeroed();
            command_request.component_id = component_id;
            command_request.schema_type = Box::into_raw(request);

            let timeout_ptr = Connection::get_option_ptr(timeout_millis);

            let command_parameters = ffi::Worker_CommandParameters {
                allow_short_circuit: 1,
            };
            let command_parameters_ptr =
                &command_parameters as *const ffi::Worker_CommandParameters;

            let request_id = ffi::Worker_Connection_SendCommandRequest(
                self.pointer,
                entity_id,
                &command_request,
                command_id,
                timeout_ptr,
                command_parameters_ptr,
            );
            Box::from_raw(command_request.schema_type);
            if !timeout_ptr.is_null() {
                Box::from_raw(timeout_ptr);
            }

            request_id
        }
    }

    pub fn send_command_response(
        &mut self,
        request_id: RequestId,
        component_id: ComponentId,
        response: Box<ffi::Schema_CommandResponse>,
    ) {
        unsafe {
            let mut command_response: ffi::Worker_CommandResponse = mem::zeroed();
            command_response.component_id = component_id;
            command_response.schema_type = Box::into_raw(response);

            Box::from_raw(command_response.schema_type);
            ffi::Worker_Connection_SendCommandResponse(self.pointer, request_id, &command_response)
        }
    }

    pub fn send_create_entity_request(
        &mut self,
        components: HashMap<ComponentId, Box<ffi::Schema_ComponentData>>,
        entity_id: Option<EntityId>,
        timeout_millis: Option<u32>,
    ) -> RequestId {
        unsafe {
            let entity_id_ptr: *mut EntityId = Connection::get_option_ptr(entity_id);
            let timeout_ptr = Connection::get_option_ptr(timeout_millis);

            let mut components: Vec<ffi::Worker_ComponentData> = components
                .into_iter()
                .map(|(component_id, data)| {
                    let mut component_data: ffi::Worker_ComponentData = mem::zeroed();
                    component_data.component_id = component_id;
                    component_data.schema_type = Box::into_raw(data);
                    component_data
                })
                .collect();

            let components_ptr = components.as_mut_ptr();

            let request_id = ffi::Worker_Connection_SendCreateEntityRequest(
                self.pointer,
                components.len() as u32,
                components_ptr,
                entity_id_ptr,
                timeout_ptr,
            );

            for component_data in components.iter() {
                Box::from_raw(component_data.schema_type);
            }

            if !entity_id_ptr.is_null() {
                Box::from_raw(entity_id_ptr);
            }
            if !timeout_ptr.is_null() {
                Box::from_raw(timeout_ptr);
            }

            request_id
        }
    }

    pub fn send_delete_entity_request(
        &mut self,
        entity_id: EntityId,
        timeout_millis: Option<u32>,
    ) -> RequestId {
        unsafe {
            let timeout_ptr = Connection::get_option_ptr(timeout_millis);
            let request_id = ffi::Worker_Connection_SendDeleteEntityRequest(
                self.pointer,
                entity_id,
                timeout_ptr,
            );
            if !timeout_ptr.is_null() {
                Box::from_raw(timeout_ptr);
            }
            request_id
        }
    }

    fn get_option_ptr<T>(value: Option<T>) -> *mut T {
        match value {
            Some(v) => Box::into_raw(Box::new(v)),
            None => ptr::null_mut(),
        }
    }
}
