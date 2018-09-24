pub struct {{COMMAND_STRUCT_NAME}};

impl Command<Schema> for {{COMMAND_STRUCT_NAME}} {
	type Component = {{COMPONENT_NAME}};
    type Request = {{REQUEST_TYPE}};
    type Response = {{RESPONSE_TYPE}};

    fn command_index() -> u32 {
    	{{COMMAND_INDEX}}
    }
}

impl CommandRequestInterface for {{REQUEST_TYPE}} {
    fn deserialise_request(request: Box<Schema_CommandRequest>) -> Self {
        unsafe {
        	let request_raw = Box::into_raw(request);
			let object = ffi::Schema_GetCommandRequestObject(request_raw);
			Box::from_raw(request_raw);

			{{REQUEST_TYPE}}::deserialise(object)
		}
    }
    fn serialise_request(&self) -> Box<Schema_CommandRequest> {
    	unsafe {
			let request = ffi::Schema_CreateCommandRequest({{COMPONENT_NAME}}::component_id(), {{COMMAND_INDEX}});
			let object = ffi::Schema_GetCommandRequestObject(request);

			self.serialise(object);

			Box::from_raw(request)
		}
    }
}

impl CommandResponseInterface for {{RESPONSE_TYPE}} {
    fn deserialise_response(response: Box<Schema_CommandResponse>) -> Self {
        unsafe {
        	let response_raw = Box::into_raw(response);
			let object = ffi::Schema_GetCommandResponseObject(response_raw);
			Box::from_raw(response_raw);

			{{RESPONSE_TYPE}}::deserialise(object)
		}
    }
    fn serialise_response(&self) -> Box<Schema_CommandResponse> {
        unsafe {
			let response = ffi::Schema_CreateCommandResponse({{COMPONENT_NAME}}::component_id(), {{COMMAND_INDEX}});
			let object = ffi::Schema_GetCommandResponseObject(response);

			self.serialise(object);

			Box::from_raw(response)
		}
    }
}
