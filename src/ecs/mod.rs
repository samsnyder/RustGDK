use specs::{World, System, Dispatcher};
use std::rc::Rc;
use std::cell::RefCell;
use worker::{View, Connection};
use schema::standard_library::Position;
use specs::{DistinctStorage, Index, UnprotectedStorage, Component};

pub struct EcsInterface<'a, 'b> {
	world: Rc<RefCell<World>>,
	view: Rc<RefCell<View<World>>>,
	dispatcher: Dispatcher<'a, 'b>
}

impl<'a, 'b> EcsInterface<'a, 'b> {
	pub fn new(connection: Connection, dispatcher: Dispatcher<'a, 'b>) -> EcsInterface<'a, 'b> {
		let world = Rc::new(RefCell::new(World::new()));
		let view = View::new(connection, world.clone());

		// world.borrow_mut().add_resource(view);
		world.borrow_mut().register::<Position>();

		view.borrow_mut().register_add_entity_callback(Box::new(|_, world, entity| {
			println!("Entity ID {}", entity.entity_id);

			world.borrow_mut().create_entity().with(Position { x: 4.0, y: 7.0 }).build();
		}));

		view.borrow_mut().register_component_update_callback(Box::new(|_, world, entity, update| {
			// use schema::standard_library::PositionUpdate;
			// match update {
			// 	PositionUpdate => expr,
			// 	_ => expr,
			// }
			// let update = update as PositionUpdate;
			// println!("Update {} {:?}", entity.entity_id, update.x);

			// world.borrow_mut().create_entity().with(Position { x: 4.0, y: 7.0 }).build();

			// world.borrow_mut().write_with_id::<Position>()
		}));

		EcsInterface {
			world,
			view,
			dispatcher
		}
	}

	pub fn process(&mut self) {
		self.view.borrow().process();
		self.dispatcher.dispatch(&self.world.borrow().res);
	}
}



impl Component for Position {
    type Storage = SpatialOSStorage<Self>;
}

pub struct SpatialOSStorage<T>{
	value: T
	// view: Rc<RefCell<View<World>>>
}

impl<T: Default> UnprotectedStorage<T> for SpatialOSStorage<T> {
    unsafe fn clean<F>(&mut self, _: F)
    where
        F: Fn(Index) -> bool,
    {
        //nothing to do
    }

    unsafe fn get(&self, id: Index) -> &T {
        &self.value
    }

    unsafe fn get_mut(&mut self, id: Index) -> &mut T {
        &mut self.value
    }

    unsafe fn insert(&mut self, id: Index, v: T) {
        // self.0.insert(id, v);
    }

    unsafe fn remove(&mut self, id: Index) -> T {
        // self.0.remove(&id).unwrap()
        Default::default()
    }
}

unsafe impl<T> DistinctStorage for SpatialOSStorage<T> {}

impl<T> Default for SpatialOSStorage<T> where T: Default,
{
    fn default() -> Self {
        SpatialOSStorage{
        	value: Default::default()
        }
    }
}
