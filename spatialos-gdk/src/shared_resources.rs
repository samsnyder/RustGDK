use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;

pub struct SharedResources {
    resources: HashMap<TypeId, Box<Any>>,
}

impl SharedResources {
    pub fn new() -> SharedResources {
        SharedResources {
            resources: HashMap::new(),
        }
    }

    pub fn add<R: 'static>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    pub fn get<R: 'static>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .map(|resource| resource.downcast_mut::<R>().unwrap())
    }
}
