use boxfnonce::BoxFnOnce;
use entity_template::EntityTemplate;
use std::any::Any;
use std::collections::HashMap;
use worker::schema::{Command, CommandRequestInterface, CommandResponseInterface, Component,
                     GeneratedSchema};
use worker::{CommandStatus, ComponentId, Connection, EntityId, RequestId};
use world::World;

type Callback<S, T> = BoxFnOnce<'static, (*mut World<S>, T, CommandStatus, String)>;
type CommandHandler<S> = Box<FnMut(&mut World<S>, &mut Connection, RequestId, EntityId, Box<Any>)>;

pub struct Commands<S: GeneratedSchema> {
    entity_command_handlers: HashMap<(ComponentId, u32), CommandHandler<S>>,
    entity_command_callbacks: HashMap<RequestId, Callback<S, (EntityId, Option<Box<Any>>)>>,
    create_entity_callbacks: HashMap<RequestId, Callback<S, EntityId>>,
    delete_entity_callbacks: HashMap<RequestId, Callback<S, EntityId>>,
}

impl<S: 'static + GeneratedSchema> Commands<S> {
    pub fn new() -> Commands<S> {
        Commands {
            entity_command_handlers: HashMap::new(),
            entity_command_callbacks: HashMap::new(),
            create_entity_callbacks: HashMap::new(),
            delete_entity_callbacks: HashMap::new(),
        }
    }

    pub fn register_handler<C: 'static + Command<S>, H: 'static>(&mut self, handler: H)
    where
        H: Fn(&mut World<S>, EntityId, &C::Request) -> C::Response,
    {
        let id = (C::Component::component_id(), C::command_index());

        if self.entity_command_handlers.contains_key(&id) {
            panic!("Command handler for component {} and command with index {} has already been registered.", id.0, id.1);
        }

        self.entity_command_handlers.insert(
            id,
            Box::new(move |world, connection, request_id, entity_id, request| {
                let request = request.downcast_ref::<C::Request>().unwrap();
                let response = handler(world, entity_id, request);
                let reponse = response.serialise_response();
                connection.send_command_response(request_id, C::Component::component_id(), reponse);
            }),
        );
    }

    pub fn on_command_request(
        &mut self,
        world: &mut World<S>,
        connection: &mut Connection,
        request_id: RequestId,
        entity_id: EntityId,
        component_id: ComponentId,
        command_id: u32,
        request: Box<Any>,
    ) {
        if let Some(handler) = self.entity_command_handlers
            .get_mut(&(component_id, command_id))
        {
            handler(world, connection, request_id, entity_id, request);
        }
    }

    pub fn send_command<C: 'static + Command<S>, A: 'static, F: 'static>(
        &mut self,
        connection: &mut Connection,
        entity_id: EntityId,
        request: C::Request,
        success: A,
        failure: F,
    ) where
        A: FnOnce(&mut World<S>, EntityId, &C::Response),
        F: FnOnce(&mut World<S>, CommandStatus, String),
    {
        let request_ptr = request.serialise_request();
        let request_id = connection.send_command_request(
            entity_id,
            C::Component::component_id(),
            request_ptr,
            C::command_index(),
            None,
        );
        Commands::<S>::register_callback(
            &mut self.entity_command_callbacks,
            request_id,
            |world, (entity_id, response)| {
                let response = response
                    .as_ref()
                    .unwrap()
                    .downcast_ref::<C::Response>()
                    .unwrap();
                success(world, entity_id, response);
            },
            failure,
        )
    }

    pub fn on_command_response(
        &mut self,
        world: &mut World<S>,
        request_id: RequestId,
        entity_id: EntityId,
        response: Option<Box<Any>>,
        success_code: CommandStatus,
        message: &str,
    ) {
        if let Some(callback) = self.entity_command_callbacks.remove(&request_id) {
            callback.call(
                world,
                (entity_id, response),
                success_code,
                message.to_string(),
            );
        }
    }

    pub fn create_entity<A: 'static, F: 'static>(
        &mut self,
        connection: &mut Connection,
        mut entity_template: EntityTemplate,
        success: A,
        failure: F,
    ) where
        A: FnOnce(&mut World<S>, EntityId),
        F: FnOnce(&mut World<S>, CommandStatus, String),
    {
        let (entity_acl_id, entity_acl_data) =
            S::serialise_entity_acl(entity_template.read_access, entity_template.write_access);
        entity_template.data.insert(entity_acl_id, entity_acl_data);

        let request_id = connection.send_create_entity_request(
            entity_template.data,
            entity_template.entity_id,
            None,
        );
        Commands::<S>::register_callback(
            &mut self.create_entity_callbacks,
            request_id,
            |world, entity_id| {
                success(world, entity_id);
            },
            failure,
        )
    }

    pub fn on_create_entity_response(
        &mut self,
        world: &mut World<S>,
        request_id: RequestId,
        entity_id: EntityId,
        success_code: CommandStatus,
        message: &str,
    ) {
        for callback in self.create_entity_callbacks.remove(&request_id) {
            callback.call(world, entity_id, success_code, message.to_string());
        }
    }

    pub fn delete_entity<A: 'static, F: 'static>(
        &mut self,
        connection: &mut Connection,
        entity_id: EntityId,
        success: A,
        failure: F,
    ) where
        A: FnOnce(&mut World<S>, EntityId),
        F: FnOnce(&mut World<S>, CommandStatus, String),
    {
        let request_id = connection.send_delete_entity_request(entity_id, None);
        Commands::<S>::register_callback(
            &mut self.delete_entity_callbacks,
            request_id,
            |world, entity_id| {
                success(world, entity_id);
            },
            failure,
        )
    }

    pub fn on_delete_entity_response(
        &mut self,
        world: &mut World<S>,
        request_id: RequestId,
        entity_id: EntityId,
        success_code: CommandStatus,
        message: &str,
    ) {
        for callback in self.delete_entity_callbacks.remove(&request_id) {
            callback.call(world, entity_id, success_code, message.to_string());
        }
    }

    fn register_callback<T: 'static, A, F>(
        callbacks: &mut HashMap<RequestId, Callback<S, T>>,
        request_id: RequestId,
        success: A,
        failure: F,
    ) where
        A: 'static + FnOnce(&mut World<S>, T),
        F: 'static + FnOnce(&mut World<S>, CommandStatus, String),
    {
        callbacks.insert(
            request_id,
            BoxFnOnce::from(move |world_ptr: *mut World<S>, object, status, message| {
                let world = unsafe { &mut (*world_ptr) };
                if status == CommandStatus::Success {
                    success(world, object);
                } else {
                    failure(world, status, message);
                }
            }),
        );
    }
}
