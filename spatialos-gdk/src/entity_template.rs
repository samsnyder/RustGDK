use std::collections::HashMap;
use worker::ffi::Schema_ComponentData;
use worker::schema::{Component, GeneratedSchema};
use worker::{ComponentId, EntityId};

/// A worker constraint for an Entity's ACL.
pub enum Worker<'a> {
    /// This specifies a worker 'class' which has the given attribute.
    /// It will match any worker with the given attribute.
    Type(&'a str),

    /// This will only match the worker with the given worker ID.
    Specific(&'a str),

    /// This can be used to match no workers.
    None,
}

impl<'a> Worker<'a> {
    fn get_attribute_string(&self) -> Option<String> {
        match self {
            &Worker::Type(attribute) => Some(String::from(attribute)),
            &Worker::Specific(id) => Some(format!("workerId:{}", id)),
            &Worker::None => None,
        }
    }
}

/// A template of a SpatialOS entity which does not exist yet.
///
/// When you create an entity using the `World`, you must give the components
/// of this new entity, along with the initial values and access control lists. These are stored in
/// an `EntityTemplate`.
///
/// The following snippet creates an `EntityTemplate` which
/// * Has it's read ACL set to `client` and `server`.
/// * Has a `Position` component with it's write ACL set to `server`.
/// * Has a `Transform` component with it's write ACL set to `server`.
///
/// ```
/// EntityBuilder::new(vec![Worker::Type("client"), Worker::Type("server")])
///     .with_component(
///         Worker::Specific("ClientWorker12345"),
///         Position {
///             coords: Coordinates {
///                 x: 0.1,
///                 y: 0.2,
///                 z: 0.3,
///             },
///         },
///     )
///     .with_component(
///         Worker::Type("server"),
///         Transform {
///             position: Vector3f {
///                 x: 0.1,
///                 y: 0.2,
///                 z: 0.3,
///             },
///         },
///     )
/// ```
pub struct EntityTemplate {
    #[doc(hidden)]
    pub entity_id: Option<EntityId>,
    #[doc(hidden)]
    pub data: HashMap<ComponentId, Box<Schema_ComponentData>>,
    #[doc(hidden)]
    pub read_access: Vec<String>,
    #[doc(hidden)]
    pub write_access: HashMap<ComponentId, String>,
}

impl EntityTemplate {

    /// Creates a new `EntityTemplate` with it's read ACL set to the
    /// union of the given `Worker` values.
    pub fn new(read_access: Vec<Worker>) -> EntityTemplate {
        EntityTemplate {
            entity_id: None,
            data: HashMap::new(),
            read_access: read_access
                .into_iter()
                .filter_map(|w| w.get_attribute_string())
                .collect(),
            write_access: HashMap::new(),
        }
    }

    /// Explicitly gives the `EntityId` which the new entity should have.
    /// This will only suceed if the given `EntityId` has already been reserved
    /// by this worker.
    pub fn set_entity_id(mut self, entity_id: EntityId) -> EntityTemplate {
        self.entity_id = Some(entity_id);
        self
    }

    /// Adds a new component to this entity, with the given write access and initial data.
    pub fn with_component<S: GeneratedSchema, C: Component<S>>(
        mut self,
        write_access: Worker,
        data: C,
    ) -> EntityTemplate {
        if let Some(attribute) = write_access.get_attribute_string() {
            self.write_access.insert(C::component_id(), attribute);
        }
        self.data
            .insert(C::component_id(), data.serialise_snapshot());
        self
    }
}
