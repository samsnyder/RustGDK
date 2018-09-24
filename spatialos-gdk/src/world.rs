use ComponentBitField;
use chunk::Chunk;
use commands::Commands;
use component_group::{Read, Write};
use entity::Entity;
use entity_collection::{Entities, EntityCollection};
use entity_template::EntityTemplate;
use shared_resources::SharedResources;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use system::System;
use worker::schema::{Command, Component, GeneratedSchema, GlobalComponentDataInterface};
use worker::{Authority, CommandStatus, ComponentId, Connection, Dispatcher, EntityId, LogLevel,
             RequestId};

/// Possible errors which can be thrown by the `World`.
pub enum WorldError {
    /// We tried to perform an operation which required a connection to SpatialOS, but
    /// the connection to SpatialOS is closed.
    ConnectionLost,
}

#[doc(hidden)]
#[derive(Default, Clone)]
pub struct WorldTime {
    timestamp: u64,
}

impl WorldTime {
    pub fn new() -> WorldTime {
        WorldTime { timestamp: 0 }
    }

    // True if a happened after b
    pub fn occured_after(&self, b: &WorldTime) -> bool {
        self.timestamp > b.timestamp
    }

    pub fn max_time<'a>(&'a self, b: &'a WorldTime) -> &'a WorldTime {
        if self.occured_after(b) {
            self
        } else {
            b
        }
    }

    pub fn get_time(&mut self) -> WorldTime {
        let time = WorldTime {
            timestamp: self.timestamp,
        };
        self.timestamp = self.timestamp + 1;
        time
    }
}

pub struct PartialEntity<S: GeneratedSchema> {
    pub entity_id: EntityId,
    pub bit_field: S::ComponentBitField,
    pub component_data: HashMap<ComponentId, S::ComponentData>,
    pub write_authority: HashMap<ComponentId, Authority>,
}

pub struct SystemData<S> {
    system: Box<System<S>>,
    last_update: WorldTime,
}

/// The `World` is the worker's view into the SpatialOS world.
///
/// It can be used to
/// * Query local entity data
/// * Send and receive commands
/// * Manage shared resources
/// * Create and delete entities
///
/// The `World` is also responsible for processing each system and each
/// SpatialOS operation. To tick the worker, you must call `process` for each tick.
///
/// ## Shared resources
///
/// You can set and retrieve shared objects of any type using `get_shared_resource`
/// and `set_shared_resource`. These objects can be used to store global information
/// or information which must be shared between systems.
pub struct World<S: GeneratedSchema> {
    connection: Connection,
    entities: EntityCollection<S>,
    added_this_cs: HashMap<EntityId, PartialEntity<S>>,
    entity_ids: HashMap<EntityId, Rc<RefCell<Entity<S>>>>,
    systems: Vec<SystemData<S>>,
    world_time: WorldTime,
    commands: Commands<S>,
    shared_resources: SharedResources,
}

impl<S: 'static + GeneratedSchema> World<S> {
    /// Construct a new `World` given an existing `Connection` to SpatialOS.
    pub fn new(connection: Connection) -> Box<World<S>> {
        let manager = Box::new(World::<S> {
            connection,
            entities: EntityCollection::new(),
            added_this_cs: HashMap::new(),
            entity_ids: HashMap::new(),
            systems: Vec::new(),
            world_time: WorldTime::new(),
            commands: Commands::new(),
            shared_resources: SharedResources::new(),
        });

        manager
    }

    /// Runs a world tick. This does the following in order:
    ///
    /// * Checks if the connection is still active.
    /// * Get's the list of ops from SpatialOS.
    /// * Processes each of these ops. This will in turn update component data and
    ///   trigger command callbacks and handlers.
    /// * Calls each registered system's `on_update` method.
    /// * Sends any updates to components which were changed by a system.
    pub fn process(&mut self, timeout_millis: u32) -> Result<(), WorldError> {
        if !self.connection.is_connected() {
            return Result::Err(WorldError::ConnectionLost);
        }

        let world_ptr = self as *mut World<S>;

        unsafe {
            let op_list = (*world_ptr).connection.get_op_list(timeout_millis);
            self.process_op_list(op_list);
        }

        unsafe {
            for system in (*world_ptr).systems.iter_mut() {
                {
                    let mut entities_view = Entities::entities_from_time(
                        &mut (*world_ptr).entities,
                        &system.last_update,
                    );
                    system.system.on_update(self, &mut entities_view);
                }
                system.last_update = self.world_time.get_time();
            }
        }

        self.entities.replicate(&mut self.connection);
        self.entities.cleanup_after_frame();

        Result::Ok(())
    }

    /// Registers a system to the World. The system's `on_ready` method will be
    /// called during this method.
    pub fn register<A: 'static + System<S> + Sized>(&mut self, mut system: A) {
        {
            system.on_ready(self);
        }
        self.systems.push(SystemData::<S> {
            system: Box::new(system),
            last_update: self.world_time.get_time(),
        });
    }

    /// Sends a log message to SpatialOS, as well as logging it to `stdout`.
    ///
    /// It is invalid to call this method if the connection is no longer active.
    pub fn log(&mut self, level: LogLevel, logger_name: &str, message: &str) {
        println!("{:?} [{}] {}", level, logger_name, message);
        self.connection
            .send_log_message(level, String::from(logger_name), String::from(message));
    }

    /// Get's the shared resource of type `R`, if it exists.
    pub fn get_shared_resource<R: 'static>(&mut self) -> Option<&mut R> {
        self.shared_resources.get::<R>()
    }

    /// Sets or replaces the shared resource of type `R`.
    pub fn set_shared_resource<R: 'static>(&mut self, resource: R) {
        self.shared_resources.add(resource)
    }

    /// Gets an immutable reference to the component data of the given `EntityId` for component `C`.
    pub fn get_component<C: 'static + Component<S>>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<Read<S, C>> {
        if let Some(entity) = self.entity_ids.get(&entity_id) {
            let entity = entity.borrow();
            self.entities
                .get_chunk_for_entity(&entity)
                .get_component_storage::<C>()
                .map(|storage| {
                    Read::new(&storage.get_component_data_entry(entity.index_in_chunk).data)
                })
        } else {
            None
        }
    }

    /// Gets a mutable reference to the component data of the given `EntityId` for component `C`.
    ///
    /// Any changes made to the component will be replicated over the network at the end of the
    /// current tick.
    pub fn get_mut_component<C: 'static + Component<S>>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<Write<S, C>> {
        if let Some(entity) = self.entity_ids.get(&entity_id) {
            let entity = entity.borrow();
            let chunk = self.entities.get_chunk_for_entity(&entity);
            chunk.mark_component_storage_as_dirty::<C>();
            chunk.get_component_storage::<C>().and_then(|storage| {
                match storage.get_authority(entity.index_in_chunk) {
                    Authority::NotAuthoritative => None,
                    _ => Some(Write::new(
                        &mut storage.get_component_data_entry(entity.index_in_chunk).data,
                    )),
                }
            })
        } else {
            None
        }
    }

    /// Gets the current authority of component `C` for the given `EntityId`.
    pub fn get_authority<C: 'static + Component<S>>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<Authority> {
        if let Some(entity) = self.entity_ids.get(&entity_id) {
            let entity = entity.borrow();
            self.entities
                .get_chunk_for_entity(&entity)
                .get_component_storage::<C>()
                .map(|storage| storage.get_authority(entity.index_in_chunk))
        } else {
            None
        }
    }

    /// Registers a command handler for command `C`. The handler takes as arguments:
    ///
    /// * A reference to this `World`.
    /// * The `EntityId` of the entity which is receiving this command.
    /// * The command request object itself.
    ///
    /// The handler must return a response which will be sent back to the calling entity.
    ///
    /// These handlers are called in a single threaded environment, outside of any system
    /// update call.
    ///
    /// ## Example
    ///
    /// ```
    /// world.register_command_handler(
    ///     Transform::example_command(),
    ///     |world, entity_id, request| {
    ///         println!("Got request: {} {:?}", entity_id, request);
    ///
    ///         ExampleResponse { reply: 0.1 }
    ///     },
    /// );
    /// ```
    pub fn register_command_handler<C: 'static + Command<S>, H: 'static>(
        &mut self,
        _command: C,
        handler: H,
    ) where
        H: Fn(&mut World<S>, EntityId, &C::Request) -> C::Response,
    {
        self.commands.register_handler::<C, H>(handler);
    }

    /// Sends a command of type `C` to the given `EntityId`. Two closures must also be given
    /// to handle the success and failure of the command.
    ///
    /// `success` is triggered if the command response was successfully received, and it takes as arguments:
    /// * A reference to this `World`.
    /// * The `EntityId` of the entity which this command was sent to.
    /// * The command response object.
    ///
    /// `failure` is triggered if there was an error sending the command, and it takes as arguments:
    /// * A reference to this `World`.
    /// * The failure code.
    /// * The failure error message.
    ///
    /// Short circuiting is enabled for this command.
    ///
    /// ## Example
    ///
    /// ```
    /// world.send_command(
    ///     Transform::example_command(),
    ///     100,
    ///     ExampleRequest { param: 0.5 },
    ///     |_world, entity_id, response| {
    ///         println!("Command succeeded: {} {:?}", entity_id, response.reply);
    ///     },
    ///     |_world, status, message| {
    ///         println!("Command failed: {:?} {}", status, message);
    ///     },
    /// );
    /// ```
    pub fn send_command<C: 'static + Command<S>, A: 'static, F: 'static>(
        &mut self,
        _command: C,
        entity_id: EntityId,
        request: C::Request,
        success: A,
        failure: F,
    ) where
        A: FnOnce(&mut World<S>, EntityId, &C::Response),
        F: FnOnce(&mut World<S>, CommandStatus, String),
    {
        self.commands.send_command::<C, A, F>(
            &mut self.connection,
            entity_id,
            request,
            success,
            failure,
        );
    }

    /// Creates a new SpatialOS entity. Two closures must also be given
    /// to handle the success and failure of this creation.
    ///
    /// `success` is triggered if the entity was successfully created, and it takes as arguments:
    /// * A reference to this `World`.
    /// * The `EntityId` of the created entity.
    ///
    /// `failure` is triggered if there was an error creating the entity, and it takes as arguments:
    /// * A reference to this `World`.
    /// * The failure code.
    /// * The failure error message.
    ///
    /// ## Example
    ///
    /// ```
    /// world.create_entity(
    ///     EntityBuilder::new(vec![Worker::Type("visual"), Worker::Type("physics")])
    ///         .with_component(
    ///             Worker::Type("physics"),
    ///             Position {
    ///                 coords: Coordinates { x: 0.1, y: 0.2, z: 0.3 },
    ///             },
    ///         )
    ///         .with_component(
    ///             Worker::Specific(client_worker_id),
    ///             Character {
    ///                 health: 50,
    ///             },
    ///         ),
    ///     |_world, entity_id| {
    ///         println!("Created entity: {}", entity_id);
    ///     },
    ///     |_world, status, message| {
    ///         println!("Failure creating entity: {:?} {}", status, message);
    ///     },
    /// );
    /// ```
    pub fn create_entity<A: 'static, F: 'static>(
        &mut self,
        entity_template: EntityTemplate,
        success: A,
        failure: F,
    ) where
        A: FnOnce(&mut World<S>, EntityId),
        F: FnOnce(&mut World<S>, CommandStatus, String),
    {
        self.commands
            .create_entity(&mut self.connection, entity_template, success, failure);
    }

    /// Deletes an existing SpatialOS entity. Two closures must also be given
    /// to handle the success and failure of this deletion.
    ///
    /// `success` is triggered if the entity was successfully deleted, and it takes as arguments:
    /// * A reference to this `World`.
    /// * The `EntityId` of the deleted entity.
    ///
    /// `failure` is triggered if there was an error deleting the entity, and it takes as arguments:
    /// * A reference to this `World`.
    /// * The failure code.
    /// * The failure error message.
    ///
    /// ## Example
    ///
    /// ```
    /// world.delete_entity(
    ///     entity_to_delete,
    ///     |_world, entity_id| {
    ///         println!("Entity {} successfully deleted.", entity_id);
    ///     },
    ///     |_world, status, message| {
    ///         println!("Failure deleting entity: {:?} {}", status, message);
    ///     },
    /// );
    /// ```
    pub fn delete_entity<A: 'static, F: 'static>(
        &mut self,
        entity_id: EntityId,
        success: A,
        failure: F,
    ) where
        A: FnOnce(&mut World<S>, EntityId),
        F: FnOnce(&mut World<S>, CommandStatus, String),
    {
        self.commands
            .delete_entity(&mut self.connection, entity_id, success, failure);
    }
}

impl<S: 'static + GeneratedSchema> Dispatcher<S> for World<S> {
    fn on_critical_section(&mut self, in_critical_section: bool) {
        if !in_critical_section {
            for (entity_id, entity) in self.added_this_cs.drain() {
                let chunk: &mut Chunk<S> =
                    self.entities
                        .get_free_chunk(&entity.bit_field, &mut self.world_time, &entity);
                let new_entity = chunk.add_entity(&mut self.world_time, entity);
                self.entity_ids.insert(entity_id, new_entity);
            }
        }
    }

    fn on_add_entity(&mut self, entity_id: EntityId) {
        self.added_this_cs.insert(
            entity_id,
            PartialEntity {
                entity_id,
                bit_field: ComponentBitField::new(),
                component_data: HashMap::new(),
                write_authority: HashMap::new(),
            },
        );
    }

    fn on_remove_entity(&mut self, entity_id: EntityId) {
        {
            let entity = self.entity_ids[&entity_id].borrow();
            let chunk = self.entities.get_chunk_for_entity(&entity);
            chunk.remove_entity(&entity);
        }

        self.entity_ids.remove(&entity_id);
    }

    fn on_add_component(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        data: S::ComponentData,
    ) {
        let entity: &mut PartialEntity<S> = self.added_this_cs.get_mut(&entity_id).unwrap();
        if entity.bit_field.add_component(component_id) {
            // We have this component
            entity.component_data.insert(component_id, data);
        }
    }

    fn on_component_update(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        update: S::ComponentUpdate,
    ) {
        if let Some(ref mut entity) = self.added_this_cs.get_mut(&entity_id) {
            let data: &mut S::ComponentData =
                &mut entity.component_data.get_mut(&component_id).unwrap();
            data.apply_update(&update);
        } else {
            let entity = self.entity_ids[&entity_id].borrow();
            let chunk = self.entities.get_chunk_for_entity(&entity);
            chunk.apply_component_update(component_id, &mut self.world_time, &entity, update);
        }
    }

    fn on_authority_change(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        authority: Authority,
    ) {
        if let Some(ref mut entity) = self.added_this_cs.get_mut(&entity_id) {
            entity.write_authority.insert(component_id, authority);
        } else {
            let entity = self.entity_ids[&entity_id].borrow();
            let chunk = self.entities.get_chunk_for_entity(&entity);
            chunk.apply_authority(component_id, &entity, authority);
        }
    }

    fn on_create_entity_response(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        status_code: CommandStatus,
        message: &str,
    ) {
        // Give responder mutable access to World as all command responses
        // happen in a single threaded environment.
        let world_ptr = self as *mut World<S>;
        let world = unsafe { &mut (*world_ptr) };

        self.commands
            .on_create_entity_response(world, request_id, entity_id, status_code, message);
    }

    fn on_delete_entity_response(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        status_code: CommandStatus,
        message: &str,
    ) {
        // Give responder mutable access to World as all command responses
        // happen in a single threaded environment.
        let world_ptr = self as *mut World<S>;
        let world = unsafe { &mut (*world_ptr) };

        self.commands
            .on_delete_entity_response(world, request_id, entity_id, status_code, message);
    }

    fn on_command_request(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        component_id: ComponentId,
        command_id: u32,
        request: Box<Any>,
    ) {
        // Give handler mutable access to World as all command handlers
        // happen in a single threaded environment.
        let world_ptr = self as *mut World<S>;
        let world = unsafe { &mut (*world_ptr) };
        self.commands.on_command_request(
            world,
            &mut self.connection,
            request_id,
            entity_id,
            component_id,
            command_id,
            request,
        );
    }

    fn on_command_response(
        &mut self,
        request_id: RequestId,
        entity_id: EntityId,
        response: Option<Box<Any>>,
        status_code: CommandStatus,
        message: &str,
    ) {
        // Give responder mutable access to World as all command responses
        // happen in a single threaded environment.
        let world_ptr = self as *mut World<S>;
        let world = unsafe { &mut (*world_ptr) };

        self.commands.on_command_response(
            world,
            request_id,
            entity_id,
            response,
            status_code,
            message,
        );
    }
}
