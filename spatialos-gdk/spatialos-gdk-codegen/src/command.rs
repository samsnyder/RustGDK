use json::{CommandDefinition, ComponentDefinition};
use quote::Tokens;
use schema_type::{ReferencedUserType, Type};
use syn::Ident;

fn snake_to_title_case(snake_case: String) -> String {
    snake_case
        .split("_")
        .flat_map(|word| {
            let mut v: Vec<char> = word.chars().collect();
            v[0] = v[0].to_uppercase().nth(0).unwrap();
            v
        })
        .collect()
}

pub struct Command {
    name: String,
    command_index: u32,
    pub request_type: ReferencedUserType,
    pub response_type: ReferencedUserType,
    struct_name: String,
    component_name: String,
    component_id: u32,
}

impl From<(CommandDefinition, ComponentDefinition)> for Command {
    fn from(value: (CommandDefinition, ComponentDefinition)) -> Command {
        let (command, component) = value;
        let struct_name = format!(
            "{}{}",
            component.name,
            snake_to_title_case(command.name.clone())
        );
        Command {
            name: command.name,
            command_index: command.commandIndex,
            request_type: ReferencedUserType::from(command.requestType),
            response_type: ReferencedUserType::from(command.responseType),
            struct_name: struct_name,
            component_name: component.name,
            component_id: component.id,
        }
    }
}

impl Command {
    pub fn getter_code(&self) -> Tokens {
        let name = Ident::new(self.name.as_str());
        let struct_name = Ident::new(self.struct_name.as_str());
        quote!{
            pub fn #name() -> #struct_name {
                #struct_name {}
            }
        }
    }

    pub fn get_global_request_match(&self) -> Tokens {
        let component_id = self.component_id;
        let command_index = self.command_index;
        let command_request_qualified_name =
            Ident::new(self.request_type.rust_qualified_name().as_str());
        quote!{
            (#component_id, #command_index) =>
                Some(Box::new(#command_request_qualified_name::deserialise_request(request)))
        }
    }

    pub fn get_global_response_match(&self) -> Tokens {
        let component_id = self.component_id;
        let command_index = self.command_index;
        let command_response_qualified_name =
            Ident::new(self.response_type.rust_qualified_name().as_str());
        quote!{
            (#component_id, #command_index) =>
                Some(Box::new(#command_response_qualified_name::deserialise_response(response)))
        }
    }

    pub fn get_code(&self) -> Tokens {
        let struct_name = Ident::new(self.struct_name.as_str());
        let component_name = Ident::new(self.component_name.as_str());
        let command_index = self.command_index;
        let request_type_name = Ident::new(self.request_type.rust_qualified_name().as_str());
        let response_type_name = Ident::new(self.response_type.rust_qualified_name().as_str());

        quote!{
            pub struct #struct_name;

            impl Command<Schema> for #struct_name {
                type Component = #component_name;
                type Request = #request_type_name;
                type Response = #response_type_name;

                fn command_index() -> u32 {
                    #command_index
                }
            }

            impl CommandRequestInterface for #request_type_name {
                fn deserialise_request(request: Box<Schema_CommandRequest>) -> Self {
                    unsafe {
                        let request_raw = Box::into_raw(request);
                        let object = ffi::Schema_GetCommandRequestObject(request_raw);
                        Box::from_raw(request_raw);

                        #request_type_name::deserialise(object)
                    }
                }
                fn serialise_request(&self) -> Box<Schema_CommandRequest> {
                    unsafe {
                        let request = ffi::Schema_CreateCommandRequest(#component_name::component_id(), #command_index);
                        let object = ffi::Schema_GetCommandRequestObject(request);

                        self.serialise(object);

                        Box::from_raw(request)
                    }
                }
            }

            impl CommandResponseInterface for #response_type_name {
                fn deserialise_response(response: Box<Schema_CommandResponse>) -> Self {
                    unsafe {
                        let response_raw = Box::into_raw(response);
                        let object = ffi::Schema_GetCommandResponseObject(response_raw);
                        Box::from_raw(response_raw);

                        #response_type_name::deserialise(object)
                    }
                }
                fn serialise_response(&self) -> Box<Schema_CommandResponse> {
                    unsafe {
                        let response = ffi::Schema_CreateCommandResponse(#component_name::component_id(), #command_index);
                        let object = ffi::Schema_GetCommandResponseObject(response);

                        self.serialise(object);

                        Box::from_raw(response)
                    }
                }
            }

        }
    }
}
