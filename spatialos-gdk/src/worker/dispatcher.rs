use std::any::Any;
use std::ffi::CStr;
use worker::ffi::{Schema_GetCommandRequestCommandIndex, Schema_GetCommandResponseCommandIndex};
use worker::schema::{GeneratedSchema, GlobalComponentDataInterface, GlobalComponentUpdateInterface};
use worker::{Authority, CommandStatus, ComponentId, EntityId, FFIEnum, Op, OpList, RequestId};

#[allow(unused_variables)]
pub trait Dispatcher<S: GeneratedSchema> {
    fn process_op_list(&mut self, op_list: OpList) {
        for op in &op_list.ops {
            match op {
                Op::CriticalSection(op) => {
                    let in_critical_section = (*op).in_critical_section == 1;
                    self.on_critical_section(in_critical_section);
                }
                Op::AddEntity(op) => self.on_add_entity((*op).entity_id),
                Op::RemoveEntity(op) => self.on_remove_entity((*op).entity_id),
                Op::CreateEntityResponse(op) => {
                    let request_id = (*op).request_id;
                    let entity_id = (*op).entity_id;
                    unsafe {
                        let status_code = CommandStatus::from_u8((*op).status_code);
                        let message = CStr::from_ptr((*op).message).to_str().unwrap();
                        self.on_create_entity_response(request_id, entity_id, status_code, message);
                    }
                }
                Op::DeleteEntityResponse(op) => {
                    let request_id = (*op).request_id;
                    let entity_id = (*op).entity_id;
                    unsafe {
                        let status_code = CommandStatus::from_u8((*op).status_code);
                        let message = CStr::from_ptr((*op).message).to_str().unwrap();
                        self.on_delete_entity_response(request_id, entity_id, status_code, message);
                    }
                }
                Op::AddComponent(op) => {
                    let entity_id = (*op).entity_id;
                    let component_id = (*op).data.component_id;
                    unsafe {
                        let component_data = Box::from_raw((*op).data.schema_type);

                        if let Some(data) =
                            S::ComponentData::deserialise(component_id, component_data)
                        {
                            self.on_add_component(entity_id, component_id, data);
                        }
                    }
                }
                Op::AuthorityChange(op) => {
                    let entity_id = (*op).entity_id;
                    let component_id = (*op).component_id;
                    unsafe {
                        let authority = Authority::from_u8((*op).authority);
                        self.on_authority_change(entity_id, component_id, authority);
                    }
                }
                Op::ComponentUpdate(op) => {
                    let entity_id = (*op).entity_id;
                    let component_id = (*op).update.component_id;
                    unsafe {
                        let component_update = Box::from_raw((*op).update.schema_type);

                        if let Some(update) =
                            S::ComponentUpdate::deserialise(component_id, component_update)
                        {
                            self.on_component_update(entity_id, component_id, update);
                        }
                    }
                }
                Op::CommandRequest(op) => {
                    let request_id = (*op).request_id;
                    let entity_id = (*op).entity_id;
                    let component_id = (*op).request.component_id;
                    let command_request_raw = (*op).request.schema_type;
                    unsafe {
                        let command_index =
                            Schema_GetCommandRequestCommandIndex(command_request_raw);
                        let command_request = Box::from_raw(command_request_raw);
                        if let Some(request) = S::deserialise_command_request(
                            component_id,
                            command_index,
                            command_request,
                        ) {
                            self.on_command_request(
                                request_id,
                                entity_id,
                                component_id,
                                command_index,
                                request,
                            );
                        }
                    }
                }
                Op::CommandResponse(op) => {
                    let request_id = (*op).request_id;
                    let entity_id = (*op).entity_id;
                    let component_id = (*op).response.component_id;
                    let command_response_raw = (*op).response.schema_type;
                    unsafe {
                        let status_code = CommandStatus::from_u8((*op).status_code);
                        let response = if command_response_raw.is_null() {
                            None
                        } else {
                            let command_index =
                                Schema_GetCommandResponseCommandIndex(command_response_raw);
                            let command_response = Box::from_raw(command_response_raw);
                            S::deserialise_command_response(
                                component_id,
                                command_index,
                                command_response,
                            )
                        };

                        let message = CStr::from_ptr((*op).message).to_str().unwrap();
                        self.on_command_response(
                            request_id,
                            entity_id,
                            response,
                            status_code,
                            message,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn on_critical_section(&mut self, in_critical_section: bool) {}
    fn on_add_entity(&mut self, entity_id: EntityId) {}
    fn on_remove_entity(&mut self, entity_id: EntityId) {}
    fn on_add_component(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        data: S::ComponentData,
    ) {
    }
    fn on_component_update(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        update: S::ComponentUpdate,
    ) {
    }
    fn on_authority_change(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        authority: Authority,
    ) {
    }
    fn on_create_entity_response(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        status_code: CommandStatus,
        message: &str,
    ) {
    }
    fn on_delete_entity_response(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        status_code: CommandStatus,
        message: &str,
    ) {
    }
    fn on_command_request(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        component_id: ComponentId,
        command_id: u32,
        request: Box<Any>,
    ) {
    }
    fn on_command_response(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        response: Option<Box<Any>>,
        status_code: CommandStatus,
        message: &str,
    ) {
    }
}
