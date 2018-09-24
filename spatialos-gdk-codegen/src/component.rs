use command::Command;
use event::Event;
use json::ComponentDefinition;
use quote::Tokens;
use schema_type::{ReferencedUserType};
use syn::Ident;
use to_rust_qualified_name;

pub struct Component {
    name: String,
    pub qualified_name: Vec<String>,
    pub rust_qualified_name: String,
    pub data_reference_type: ReferencedUserType,
    pub events: Vec<Event>,
    pub commands: Vec<Command>,
    pub component_id: u32,
}

impl From<ComponentDefinition> for Component {
    fn from(value: ComponentDefinition) -> Component {
        let command_component_value = value.clone();
        Component {
            name: value.name,
            component_id: value.id,
            qualified_name: value
                .qualifiedName
                .split(".")
                .map(|s| String::from(s))
                .collect(),
            rust_qualified_name: to_rust_qualified_name(value.qualifiedName.as_str()),
            data_reference_type: ReferencedUserType::from(value.dataDefinition),
            events: {
                value
                    .eventDefinitions
                    .into_iter()
                    .map(|event_def| Event::from(event_def))
                    .collect()
            },
            commands: {
                value
                    .commandDefinitions
                    .into_iter()
                    .map(|command_def| {
                        Command::from((command_def, command_component_value.clone()))
                    })
                    .collect()
            },
        }
    }
}

impl Component {
    pub fn enum_name(&self) -> String {
        format!("Component{}", self.component_id)
    }

    pub fn get_code(&self) -> Tokens {
        let name = Ident::new(self.name.as_str());
        let data_name = Ident::new(format!("{}Data", &self.name));
        let update_name = Ident::new(format!("{}Update", &self.name));
        let component_id = self.component_id;
        let enum_name = Ident::new(self.enum_name().as_str());

        let fields = &self.data_reference_type.get().unwrap().borrow().fields;

        let field_definitions = fields.iter().map(|field| field.definition_in_struct(false));
        let field_data_definitions = fields.iter().map(|field| field.definition_in_struct(true));
        let event_data_definitions = self.events.iter().map(|event| event.definition_in_struct());
        let field_serialise_from_data = fields.iter().map(|field| field.serialise_from_data());
        let field_deserialise_into_data = fields.iter().map(|field| field.deserialise_into_data());
        let event_initial_code = self.events.iter().map(|event| event.initial_code());
        let field_update_definitions = fields.iter().map(|field| field.definition_in_update());
        let event_update_definitions = self.events.iter().map(|event| event.definition_in_update());
        let field_apply_from_update = fields.iter().map(|field| field.apply_from_update());
        let event_apply_from_update = self.events.iter().map(|event| event.apply_from_update());
        let field_serialise_from_dirty_data =
            fields.iter().map(|field| field.serialise_from_dirty_data());
        let event_serialise_from_dirty_data = self.events
            .iter()
            .map(|event| event.serialise_from_dirty_data());
        let field_deserialise_into_update =
            fields.iter().map(|field| field.deserialise_into_update());
        let event_deserialise_into_update = self.events
            .iter()
            .map(|event| event.deserialise_into_update());
        let snapshot_to_data_fields = fields.iter().map(|field| field.snapshot_to_data());
        let snapshot_event_initial_code = self.events.iter().map(|event| event.initial_code());
        let clear_events = self.events.iter().map(|event| event.clear_events_code());
        let contains_events = self.events.iter().map(|event| event.contains_events_code());
        let command_getters = self.commands.iter().map(|command| command.getter_code());
        let command_code = self.commands.iter().map(|command| command.get_code());

        quote!{
            #[allow(dead_code, unused_variables)]
            #[derive(Default)]
            pub struct #name{
                #(#field_definitions,)*
            }

            #[allow(dead_code, unused_variables)]
            impl #name {
                #(#command_getters)*
            }

            #(#command_code)*

            #[allow(dead_code, unused_variables)]
            impl Component<Schema> for #name {
                type Data = #data_name;
                type Update = #update_name;

                fn component_id() -> ComponentId {
                    #component_id
                }

                fn apply_update_to_data(data: &mut Self::Data, update: &Self::Update) {
                    data.apply_update(update);
                }

                fn extract_data_borrow(data: &<Schema as GeneratedSchema>::ComponentData) -> Option<&Self::Data> {
                    match data {
                        &ComponentData::#enum_name(ref data) => Some(data),
                        _ => None
                    }
                }

                fn extract_data(data: <Schema as GeneratedSchema>::ComponentData) -> Option<Self::Data> {
                    match data {
                        ComponentData::#enum_name(data) => Some(data),
                        _ => None
                    }
                }

                fn extract_update(update: &<Schema as GeneratedSchema>::ComponentUpdate) -> Option<&Self::Update> {
                    match update {
                        &ComponentUpdate::#enum_name(ref update) => Some(update),
                        _ => None
                    }
                }

                fn serialise_snapshot(self) -> Box<ffi::Schema_ComponentData> {
                    let data = #data_name {
                        is_dirty: false,
                        #(#snapshot_to_data_fields,)*
                        #(#snapshot_event_initial_code,)*
                    };
                    data.serialise_data()
                }
            }

            #[allow(dead_code, unused_variables)]
            #[derive(Clone, Debug)]
            pub struct #update_name {
                #(#field_update_definitions,)*
                #(#event_update_definitions,)*
            }

            #[allow(dead_code, unused_variables)]
            #[derive(Clone, Debug, Default)]
            pub struct #data_name {
                is_dirty: bool,
                #(#field_data_definitions,)*
                #(#event_data_definitions,)*
            }

            #[allow(dead_code, unused_variables)]
            impl #data_name {
                pub unsafe fn deserialise(object: *mut Schema_Object) -> #data_name {
                    #data_name {
                        is_dirty: false,
                        #(#field_deserialise_into_data,)*
                        #(#event_initial_code,)*
                    }
                }

                pub unsafe fn serialise(&self, object: *mut Schema_Object) {
                    #(#field_serialise_from_data;)*
                }
            }

            #[allow(dead_code, unused_variables)]
            impl #data_name {

                pub fn apply_update(&mut self, update: &#update_name) {
                    #(#field_apply_from_update;)*

                    #(#event_apply_from_update;)*
                }
            }

            #[allow(dead_code, unused_variables)]
            impl ComponentDataInterface<Schema> for #data_name {
                fn deserialise_data(data: Box<ffi::Schema_ComponentData>) -> <Schema as GeneratedSchema>::ComponentData {
                    unsafe {
                        let data_raw = Box::into_raw(data);
                        let fields = ffi::Schema_GetComponentDataFields(data_raw);
                        Box::from_raw(data_raw);

                        ComponentData::#enum_name(#data_name::deserialise(fields))
                    }
                }

                fn serialise_data(&self) -> Box<ffi::Schema_ComponentData> {
                    unsafe {
                        let data = ffi::Schema_CreateComponentData(#name::component_id());
                        let fields = ffi::Schema_GetComponentDataFields(data);

                        self.serialise(fields);

                        Box::from_raw(data)
                    }
                }

                fn serialise_update(&mut self) -> Box<ffi::Schema_ComponentUpdate> {
                    unsafe {
                        let update = ffi::Schema_CreateComponentUpdate(#name::component_id());
                        let fields = ffi::Schema_GetComponentUpdateFields(update);
                        let events = ffi::Schema_GetComponentUpdateEvents(update);

                        #(#field_serialise_from_dirty_data)*

                        #(#event_serialise_from_dirty_data)*

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
                    #(#clear_events;)*
                }
            }

            #[allow(dead_code, unused_variables)]
            impl ComponentUpdateInterface<Schema> for #update_name {
                fn deserialise_update(update_box: Box<ffi::Schema_ComponentUpdate>) -> <Schema as GeneratedSchema>::ComponentUpdate {
                    unsafe {
                        let update = Box::into_raw(update_box);
                        let fields = ffi::Schema_GetComponentUpdateFields(update);
                        Box::from_raw(update);

                        ComponentUpdate::#enum_name(#update_name {
                            #(#field_deserialise_into_update,)*
                            #(#event_deserialise_into_update,)*
                        })
                    }
                }

                fn contains_events(&self) -> bool {
                    #(#contains_events ||)* false
                }
            }
        }
    }
}
