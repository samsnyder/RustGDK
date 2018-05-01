use worker::{EntityId, ComponentId};

pub trait ComponentMetaclass {
	fn component_id() -> ComponentId;
}

pub trait ComponentUpdate {
}