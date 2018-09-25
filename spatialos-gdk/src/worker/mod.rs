mod connection;
mod dispatcher;
pub mod ffi;
pub mod schema;
mod snapshot;

pub use self::connection::{Connection, ConnectionParameters};
pub use self::dispatcher::Dispatcher;
pub use self::snapshot::SnapshotOutputStream;

use std::slice;

pub type EntityId = i64;
pub type ComponentId = u32;
pub type RequestId = u32;

pub trait FFIEnum: Sized + Copy {
    unsafe fn get_u8(self) -> u8 {
        *(&self as *const _ as *const u8)
    }

    unsafe fn from_u8(value: u8) -> Self {
        *(&value as *const _ as *const Self)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum LogLevel {
    Debug = ffi::Worker_LogLevel::WORKER_LOG_LEVEL_DEBUG as u8,
    Info = ffi::Worker_LogLevel::WORKER_LOG_LEVEL_INFO as u8,
    Warning = ffi::Worker_LogLevel::WORKER_LOG_LEVEL_WARN as u8,
    Error = ffi::Worker_LogLevel::WORKER_LOG_LEVEL_ERROR as u8,
    Fatal = ffi::Worker_LogLevel::WORKER_LOG_LEVEL_FATAL as u8,
}
impl FFIEnum for LogLevel {}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum Authority {
    NotAuthoritative = ffi::Worker_Authority::WORKER_AUTHORITY_NOT_AUTHORITATIVE as u8,
    Authoritative = ffi::Worker_Authority::WORKER_AUTHORITY_AUTHORITATIVE as u8,
    AuthorityLossImminent = ffi::Worker_Authority::WORKER_AUTHORITY_AUTHORITY_LOSS_IMMINENT as u8,
}
impl FFIEnum for Authority {}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum CommandStatus {
    Success = ffi::Worker_StatusCode::WORKER_STATUS_CODE_SUCCESS as u8,
    Timeout = ffi::Worker_StatusCode::WORKER_STATUS_CODE_TIMEOUT as u8,
    NotFound = ffi::Worker_StatusCode::WORKER_STATUS_CODE_NOT_FOUND as u8,
    AuthorityLost = ffi::Worker_StatusCode::WORKER_STATUS_CODE_AUTHORITY_LOST as u8,
    PermissionDenied = ffi::Worker_StatusCode::WORKER_STATUS_CODE_PERMISSION_DENIED as u8,
    ApplicationError = ffi::Worker_StatusCode::WORKER_STATUS_CODE_APPLICATION_ERROR as u8,
    InternalError = ffi::Worker_StatusCode::WORKER_STATUS_CODE_INTERNAL_ERROR as u8,
}
impl FFIEnum for CommandStatus {}

#[repr(u8)]
pub enum Op<'a> {
    Disconnect(&'a ffi::Worker_DisconnectOp),
    FlagUpdate(&'a ffi::Worker_FlagUpdateOp),
    LogMessage(&'a ffi::Worker_LogMessageOp),
    Metrics(&'a ffi::Worker_MetricsOp),
    CriticalSection(&'a ffi::Worker_CriticalSectionOp),
    AddEntity(&'a ffi::Worker_AddEntityOp),
    RemoveEntity(&'a ffi::Worker_RemoveEntityOp),
    ReserveEntityIdResponse(&'a ffi::Worker_ReserveEntityIdResponseOp),
    ReserveEntityIdsResponse(&'a ffi::Worker_ReserveEntityIdsResponseOp),
    CreateEntityResponse(&'a ffi::Worker_CreateEntityResponseOp),
    DeleteEntityResponse(&'a ffi::Worker_DeleteEntityResponseOp),
    EntityQueryResponse(&'a ffi::Worker_EntityQueryResponseOp),
    AddComponent(&'a ffi::Worker_AddComponentOp),
    RemoveComponent(&'a ffi::Worker_RemoveComponentOp),
    AuthorityChange(&'a ffi::Worker_AuthorityChangeOp),
    ComponentUpdate(&'a ffi::Worker_ComponentUpdateOp),
    CommandRequest(&'a ffi::Worker_CommandRequestOp),
    CommandResponse(&'a ffi::Worker_CommandResponseOp),
    Unknown,
}

#[cfg(windows)]
type BindegenEnumType = i32;

#[cfg(not(windows))]
type BindegenEnumType = u32;

impl<'a> Op<'a> {
    fn from_union(worker_op: &ffi::Worker_Op) -> Op {
        unsafe {
            match worker_op.op_type as BindegenEnumType {
                ffi::Worker_OpType::WORKER_OP_TYPE_DISCONNECT => {
                    Op::Disconnect(worker_op.__bindgen_anon_1.disconnect.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_FLAG_UPDATE => {
                    Op::FlagUpdate(worker_op.__bindgen_anon_1.flag_update.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_LOG_MESSAGE => {
                    Op::LogMessage(worker_op.__bindgen_anon_1.log_message.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_METRICS => {
                    Op::Metrics(worker_op.__bindgen_anon_1.metrics.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_CRITICAL_SECTION => {
                    Op::CriticalSection(worker_op.__bindgen_anon_1.critical_section.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_ADD_ENTITY => {
                    Op::AddEntity(worker_op.__bindgen_anon_1.add_entity.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_REMOVE_ENTITY => {
                    Op::RemoveEntity(worker_op.__bindgen_anon_1.remove_entity.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_RESERVE_ENTITY_ID_RESPONSE => {
                    Op::ReserveEntityIdResponse(
                        worker_op
                            .__bindgen_anon_1
                            .reserve_entity_id_response
                            .as_ref(),
                    )
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_RESERVE_ENTITY_IDS_RESPONSE => {
                    Op::ReserveEntityIdsResponse(
                        worker_op
                            .__bindgen_anon_1
                            .reserve_entity_ids_response
                            .as_ref(),
                    )
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_CREATE_ENTITY_RESPONSE => {
                    Op::CreateEntityResponse(
                        worker_op.__bindgen_anon_1.create_entity_response.as_ref(),
                    )
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_DELETE_ENTITY_RESPONSE => {
                    Op::DeleteEntityResponse(
                        worker_op.__bindgen_anon_1.delete_entity_response.as_ref(),
                    )
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_ENTITY_QUERY_RESPONSE => {
                    Op::EntityQueryResponse(
                        worker_op.__bindgen_anon_1.entity_query_response.as_ref(),
                    )
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_ADD_COMPONENT => {
                    Op::AddComponent(worker_op.__bindgen_anon_1.add_component.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_REMOVE_COMPONENT => {
                    Op::RemoveComponent(worker_op.__bindgen_anon_1.remove_component.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_AUTHORITY_CHANGE => {
                    Op::AuthorityChange(worker_op.__bindgen_anon_1.authority_change.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_COMPONENT_UPDATE => {
                    Op::ComponentUpdate(worker_op.__bindgen_anon_1.component_update.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_COMMAND_REQUEST => {
                    Op::CommandRequest(worker_op.__bindgen_anon_1.command_request.as_ref())
                }
                ffi::Worker_OpType::WORKER_OP_TYPE_COMMAND_RESPONSE => {
                    Op::CommandResponse(worker_op.__bindgen_anon_1.command_response.as_ref())
                }
                _ => Op::Unknown,
            }
        }
    }
}

pub struct OpList<'a> {
    pointer: *mut ffi::Worker_OpList,
    pub ops: Vec<Op<'a>>,
}

impl<'a> OpList<'a> {
    pub fn new(pointer: *mut ffi::Worker_OpList) -> OpList<'a> {
        unsafe {
            let ops: Vec<Op> = slice::from_raw_parts((*pointer).ops, (*pointer).op_count as usize)
                .iter()
                .map(|worker_op| Op::from_union(worker_op))
                .collect();

            OpList { pointer, ops }
        }
    }
}

impl<'a> Drop for OpList<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::Worker_OpList_Destroy(self.pointer);
        }
    }
}
