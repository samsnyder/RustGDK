use worker::EntityId;

pub struct Entity {
	pub entity_id: EntityId
}

impl Entity {
	pub fn new(entity_id: EntityId) -> Entity {
		Entity {
			entity_id
		}
	}
}