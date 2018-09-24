#[allow(dead_code, unused_variables)] 
#[derive(Clone, Debug, Default)]
pub struct {{NAME}} {
{{DEF_FIELDS}}
}

#[allow(dead_code, unused_variables)]
impl {{NAME}} {
	pub unsafe fn deserialise(object: *mut Schema_Object) -> {{NAME}} {
		{{NAME}} {
{{READ_FIELDS}}
		}
	}

	pub unsafe fn serialise(&self, object: *mut Schema_Object) {
{{WRITE_FIELDS}}
	}
}