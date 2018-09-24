#[allow(dead_code, unused_variables)]
#[derive(Default)]
pub struct {{NAME}}{
	{{DEF_FIELDS}}
}

#[allow(dead_code, unused_variables)]
impl {{NAME}} {
	{{COMMAND_GETTERS}}
}

#[allow(dead_code, unused_variables)]
impl Component<Schema> for {{NAME}} {
	type Data = {{NAME}}Data;
	type Update = {{NAME}}Update;

	fn component_id() -> ComponentId {
		{{COMPONENT_ID}}
	}

	fn apply_update_to_data(data: &mut Self::Data, update: &Self::Update) {
		data.apply_update(update);
	}

	fn extract_data_borrow(data: &<Schema as GeneratedSchema>::ComponentData) -> Option<&Self::Data> {
		match data {
			&ComponentData::{{ENUM_NAME}}(ref data) => Some(data),
			_ => None
		}
	}

	fn extract_data(data: <Schema as GeneratedSchema>::ComponentData) -> Option<Self::Data> {
		match data {
			ComponentData::{{ENUM_NAME}}(data) => Some(data),
			_ => None
		}
	}

	fn extract_update(update: &<Schema as GeneratedSchema>::ComponentUpdate) -> Option<&Self::Update> {
		match update {
			&ComponentUpdate::{{ENUM_NAME}}(ref update) => Some(update),
			_ => None
		}
	}

	fn serialise_snapshot(self) -> Box<ffi::Schema_ComponentData> {
		let data = {{NAME}}Data {
			is_dirty: false,
			{{SNAPSHOT_TO_DATA_FIELDS}}
		};
		data.serialise_data()
	}
}

#[allow(dead_code, unused_variables)] 
#[derive(Clone, Debug)]
pub struct {{NAME}}Update {
{{DEF_UPDATE_FIELDS}}
}

#[allow(dead_code, unused_variables)]
impl {{NAME}}Data {

	pub fn apply_update(&mut self, update: &{{NAME}}Update) {
{{APPLY_UPDATE_FIELDS}}

{{APPLY_UPDATE_EVENTS}}
	}
}

#[allow(dead_code, unused_variables)]
impl ComponentDataInterface<Schema> for {{NAME}}Data {
	fn deserialise_data(data: Box<ffi::Schema_ComponentData>) -> <Schema as GeneratedSchema>::ComponentData {
		unsafe {
			let data_raw = Box::into_raw(data);
			let fields = ffi::Schema_GetComponentDataFields(data_raw);
			Box::from_raw(data_raw);

			ComponentData::{{ENUM_NAME}}({{NAME}}Data::deserialise(fields))
		}
	}

	fn serialise_data(&self) -> Box<ffi::Schema_ComponentData> {
		unsafe {
			let data = ffi::Schema_CreateComponentData({{NAME}}::component_id());
			let fields = ffi::Schema_GetComponentDataFields(data);

			self.serialise(fields);

			Box::from_raw(data)
		}
	}

	fn serialise_update(&mut self) -> Box<ffi::Schema_ComponentUpdate> {
		unsafe {
			let update = ffi::Schema_CreateComponentUpdate({{NAME}}::component_id());
			let fields = ffi::Schema_GetComponentUpdateFields(update);
			let events = ffi::Schema_GetComponentUpdateEvents(update);

			{{DIRTY_UPDATE_FIELDS}}

			{{SERIALISE_UPDATE_EVENTS}}

			Box::from_raw(update)
		}
	}

	fn make_dirty(&mut self) {
		self.is_dirty = true;
	}

	fn get_and_clear_dirty_bit(&mut self) -> bool {
		let dirty = self.is_dirty;
		self.is_dirty = false;
		dirty
	}

	fn cleanup_after_frame(&mut self) {
		{{CLEAR_EVENTS}}
	}
}

#[allow(dead_code, unused_variables)]
impl ComponentUpdateInterface<Schema> for {{NAME}}Update {
	fn deserialise_update(update_box: Box<ffi::Schema_ComponentUpdate>) -> <Schema as GeneratedSchema>::ComponentUpdate {
		unsafe {
			let update = Box::into_raw(update_box);
			let fields = ffi::Schema_GetComponentUpdateFields(update);
			Box::from_raw(update);

			ComponentUpdate::{{ENUM_NAME}}({{NAME}}Update {
				{{READ_UPDATE_FIELDS}}
			})
		}
	}

	fn contains_events(&self) -> bool {
		{{CONTAINS_EVENTS}} false
	}
}