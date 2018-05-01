use specs::{Component, VecStorage};
use worker::{ComponentMetaclass, ComponentUpdate};
use worker::ComponentId;

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32
}

pub struct PositionUpdate {
    pub x: Option<f32>,
    pub y: Option<f32>
}

impl ComponentMetaclass for Position {
	fn component_id() -> ComponentId {
		54
	}
}

impl ComponentUpdate for PositionUpdate {}

impl PositionUpdate {
	pub fn deserialise() -> PositionUpdate {
		PositionUpdate {
			x: Some(3.0),
			y: Some(4.0)
		}
	}
}