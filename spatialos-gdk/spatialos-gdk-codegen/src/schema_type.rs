use json::{ListTypeDefinition, MapTypeDefinition, OptionTypeDefinition, SchemaTypeDefinition};
use quote::Tokens;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use syn::Ident;
use user_type::UserType;

pub trait Type {
    fn rust_type_name(&self) -> String;
    fn rust_qualified_name(&self) -> String;
    fn deserialise_code(
        &self,
        object_name: &String,
        field_id: u32,
        index: Option<String>,
    ) -> Tokens;
    fn serialise_code(&self, object_name: &String, field_id: u32, value_name: &String) -> Tokens;
    fn count_code(&self, object_name: &String, field_id: u32) -> Tokens {
        let object_name = Ident::new(object_name.as_str());
        quote!(ffi::Schema_GetObjectCount(#object_name, #field_id))
    }
    fn is_singular_type(&self) -> bool {
        false
    }
    fn is_built_in_type(&self) -> bool {
        false
    }
    fn got_all_types(&mut self, _: &HashMap<String, Rc<RefCell<UserType>>>) {}
}

impl From<SchemaTypeDefinition> for Box<Type> {
    fn from(value: SchemaTypeDefinition) -> Self {
        if value.builtInType.is_some() {
            Box::new(BuiltInType::from(value))
        } else if value.userType.is_some() {
            Box::new(ReferencedUserType::from(value))
        } else {
            panic!(
                "Field definition without a built in type or user type: {:?}",
                value
            );
        }
    }
}

pub struct ReferencedUserType {
    pub name: String,
    referenced_type: Option<Rc<RefCell<UserType>>>,
}

impl From<SchemaTypeDefinition> for ReferencedUserType {
    fn from(value: SchemaTypeDefinition) -> Self {
        ReferencedUserType {
            name: value.userType.unwrap(),
            referenced_type: None,
        }
    }
}

impl ReferencedUserType {
    pub fn get(&self) -> Option<&Rc<RefCell<UserType>>> {
        self.referenced_type.as_ref()
    }
}

// impl From<SchemaTypeDefinition> for ReferencedUserType {
// 	fn from(value: SchemaTypeDefinition) -> Self {
// 		ReferencedUserType {
// 			name: value.userType.unwrap(),
// 			referenced_type: None
// 		}
// 	}
// }

impl Type for ReferencedUserType {
    fn rust_type_name(&self) -> String {
        self.referenced_type
            .as_ref()
            .unwrap()
            .borrow()
            .rust_type_name()
    }

    fn rust_qualified_name(&self) -> String {
        self.referenced_type
            .as_ref()
            .unwrap()
            .borrow()
            .rust_qualified_name()
    }

    fn deserialise_code(
        &self,
        object_name: &String,
        field_id: u32,
        index: Option<String>,
    ) -> Tokens {
        self.referenced_type
            .as_ref()
            .unwrap()
            .borrow()
            .deserialise_code(object_name, field_id, index)
    }

    fn serialise_code(&self, object_name: &String, field_id: u32, value_name: &String) -> Tokens {
        self.referenced_type
            .as_ref()
            .unwrap()
            .borrow()
            .serialise_code(object_name, field_id, value_name)
    }

    fn count_code(&self, object_name: &String, field_id: u32) -> Tokens {
        self.referenced_type
            .as_ref()
            .unwrap()
            .borrow()
            .count_code(object_name, field_id)
    }

    fn got_all_types(&mut self, all_types: &HashMap<String, Rc<RefCell<UserType>>>) {
        self.referenced_type = Some(all_types[&self.name].clone());
    }
}

#[derive(PartialEq)]
pub enum BuiltInType {
    Boolean,
    UInt32,
    UInt64,
    Int32,
    Int64,
    SInt32,
    SInt64,
    Fixed32,
    Fixed64,
    SFixed32,
    SFixed64,
    Float,
    Double,
    String,
    Bytes,
    EntityId,
}

impl From<SchemaTypeDefinition> for BuiltInType {
    fn from(value: SchemaTypeDefinition) -> Self {
        match value.builtInType.unwrap().as_str() {
            "bool" => BuiltInType::Boolean,
            "uint32" => BuiltInType::UInt32,
            "uint64" => BuiltInType::UInt64,
            "int32" => BuiltInType::Int32,
            "int64" => BuiltInType::Int64,
            "sint32" => BuiltInType::SInt32,
            "sint64" => BuiltInType::SInt64,
            "fixed32" => BuiltInType::Fixed32,
            "fixed64" => BuiltInType::Fixed64,
            "sfixed32" => BuiltInType::SFixed32,
            "sfixed64" => BuiltInType::SFixed64,
            "float" => BuiltInType::Float,
            "double" => BuiltInType::Double,
            "string" => BuiltInType::String,
            "bytes" => BuiltInType::Bytes,
            "EntityId" => BuiltInType::EntityId,
            t => panic!("Unknown built in type: {:?}", t),
        }
    }
}

impl BuiltInType {
    fn type_method_group(&self) -> &str {
        match self {
            BuiltInType::Boolean => "Boolean",
            BuiltInType::UInt32 => "Uint32",
            BuiltInType::UInt64 => "Uint64",
            BuiltInType::Int32 => "Int32",
            BuiltInType::Int64 => "Int64",
            BuiltInType::SInt32 => "Sint32",
            BuiltInType::SInt64 => "Sint64",
            BuiltInType::Fixed32 => "Fixed32",
            BuiltInType::Fixed64 => "Fixed64",
            BuiltInType::SFixed32 => "Sfixed32",
            BuiltInType::SFixed64 => "Sfixed64",
            BuiltInType::Float => "Float",
            BuiltInType::Double => "Double",
            BuiltInType::String => "String",
            BuiltInType::Bytes => "BytesVec",
            BuiltInType::EntityId => "EntityId",
        }
    }
}

impl Type for BuiltInType {
    fn is_built_in_type(&self) -> bool {
        true
    }

    fn rust_type_name(&self) -> String {
        String::from(match self {
            BuiltInType::Boolean => "bool",
            BuiltInType::UInt32 => "u32",
            BuiltInType::UInt64 => "u64",
            BuiltInType::Int32 => "i32",
            BuiltInType::Int64 => "i64",
            BuiltInType::SInt32 => "i32",
            BuiltInType::SInt64 => "i64",
            BuiltInType::Fixed32 => "u32",
            BuiltInType::Fixed64 => "u64",
            BuiltInType::SFixed32 => "i32",
            BuiltInType::SFixed64 => "i64",
            BuiltInType::Float => "f32",
            BuiltInType::Double => "f64",
            BuiltInType::String => "String",
            BuiltInType::Bytes => "Vec<u8>",
            BuiltInType::EntityId => "EntityId",
        })
    }

    fn rust_qualified_name(&self) -> String {
        self.rust_type_name()
    }

    fn deserialise_code(
        &self,
        object_name: &String,
        field_id: u32,
        index: Option<String>,
    ) -> Tokens {
        let object_name = Ident::new(object_name.as_str());
        let index = Ident::new(index.unwrap_or(String::from("0")).as_str());
        let type_method_group = Ident::new(format!("Schema_Index{}", self.type_method_group()));
        quote!(ffi::#type_method_group(#object_name, #field_id, #index))
    }

    fn serialise_code(&self, object_name: &String, field_id: u32, value_name: &String) -> Tokens {
        let object_name = Ident::new(object_name.as_str());
        let type_method_group = Ident::new(format!("Schema_Add{}", self.type_method_group()));
        let mut value_name = value_name.clone();
        if *self == BuiltInType::String || *self == BuiltInType::Bytes {
            value_name = format!("&{}", value_name);
        }
        let value_name = Ident::new(value_name.as_str());
        quote!(ffi::#type_method_group(#object_name, #field_id, #value_name))
    }

    fn count_code(&self, object_name: &String, field_id: u32) -> Tokens {
        let object_name = Ident::new(object_name.as_str());
        let type_method_group = Ident::new(format!("Schema_Get{}Count", self.type_method_group()));
        quote!(ffi::#type_method_group(#object_name, #field_id))
    }
}

pub struct OptionType {
    value_type: Box<Type>,
}

impl From<OptionTypeDefinition> for OptionType {
    fn from(value: OptionTypeDefinition) -> Self {
        OptionType {
            value_type: Box::<Type>::from(value.valueType),
        }
    }
}

impl Type for OptionType {
    fn rust_type_name(&self) -> String {
        format!("Option<{}>", self.value_type.rust_type_name())
    }

    fn rust_qualified_name(&self) -> String {
        format!("Option<{}>", self.value_type.rust_qualified_name())
    }

    fn deserialise_code(&self, object_name: &String, field_id: u32, _: Option<String>) -> Tokens {
        let get_count_code = self.count_code(object_name, field_id);
        let value_deserialise_code = self.value_type
            .deserialise_code(object_name, field_id, None);
        quote!{
            {
                if #get_count_code > 0 {
                    Some(#value_deserialise_code)
                }else{
                    None
                }
            }
        }
    }

    fn serialise_code(&self, object_name: &String, field_id: u32, value_name: &String) -> Tokens {
        let value_name = Ident::new(value_name.as_str());
        let value_serialise_code =
            self.value_type
                .serialise_code(object_name, field_id, &String::from("*value"));
        quote!{
            {
                if let Some(ref value) = #value_name {
                    #value_serialise_code;
                }
            }
        }
    }

    fn got_all_types(&mut self, all_types: &HashMap<String, Rc<RefCell<UserType>>>) {
        self.value_type.got_all_types(all_types);
    }
}

pub struct ListType {
    value_type: Box<Type>,
}

impl From<ListTypeDefinition> for ListType {
    fn from(value: ListTypeDefinition) -> Self {
        ListType {
            value_type: Box::<Type>::from(value.valueType),
        }
    }
}

impl Type for ListType {
    fn rust_type_name(&self) -> String {
        format!("Vec<{}>", self.value_type.rust_type_name())
    }

    fn rust_qualified_name(&self) -> String {
        format!("Vec<{}>", self.value_type.rust_qualified_name())
    }

    fn deserialise_code(&self, object_name: &String, field_id: u32, _: Option<String>) -> Tokens {
        let rust_type_name = Ident::new(self.rust_qualified_name().as_str());
        let get_count_code = self.count_code(object_name, field_id);
        let value_deserialise_code =
            self.value_type
                .deserialise_code(object_name, field_id, Some(String::from("i")));
        quote!{
            {
                let count = #get_count_code;
                let mut list: #rust_type_name = Vec::with_capacity(count as usize);
                for i in 0..count {
                    list.push(#value_deserialise_code);
                }
                list
            }
        }
    }

    fn serialise_code(&self, object_name: &String, field_id: u32, value_name: &String) -> Tokens {
        let value_name = Ident::new(value_name.as_str());
        let value_serialise_code =
            self.value_type
                .serialise_code(object_name, field_id, &String::from("value"));
        quote!{
            {
                for ref value in #value_name.iter() {
                    #value_serialise_code;
                }
            }
        }
    }

    fn got_all_types(&mut self, all_types: &HashMap<String, Rc<RefCell<UserType>>>) {
        self.value_type.got_all_types(all_types);
    }
}

pub struct MapType {
    key_type: Box<Type>,
    value_type: Box<Type>,
}

impl From<MapTypeDefinition> for MapType {
    fn from(value: MapTypeDefinition) -> Self {
        MapType {
            key_type: Box::<Type>::from(value.keyType),
            value_type: Box::<Type>::from(value.valueType),
        }
    }
}

impl Type for MapType {
    fn rust_type_name(&self) -> String {
        format!(
            "HashMap<{}, {}>",
            self.key_type.rust_type_name(),
            self.value_type.rust_type_name()
        )
    }

    fn rust_qualified_name(&self) -> String {
        format!(
            "HashMap<{}, {}>",
            self.key_type.rust_qualified_name(),
            self.value_type.rust_qualified_name()
        )
    }

    fn deserialise_code(&self, object_name: &String, field_id: u32, _: Option<String>) -> Tokens {
        let rust_type_name = Ident::new(self.rust_qualified_name().as_str());
        let get_count_code = self.count_code(object_name, field_id);
        let object_name = Ident::new(object_name.as_str());
        let key_deserialise_code = self.key_type
            .deserialise_code(&String::from("kvp"), 1, None);
        let value_deserialise_code =
            self.value_type
                .deserialise_code(&String::from("kvp"), 2, None);
        quote!{
            {
                let count = #get_count_code;
                let mut map: #rust_type_name = HashMap::with_capacity(count as usize);
                for i in 0..count {
                    let kvp = ffi::Schema_IndexObject(#object_name, #field_id, i);
                    map.insert(#key_deserialise_code, #value_deserialise_code);
                }
                map
            }
        }
    }

    fn serialise_code(&self, object_name: &String, field_id: u32, value_name: &String) -> Tokens {
        let object_name = Ident::new(object_name.as_str());
        let value_name = Ident::new(value_name.as_str());
        let key_serialise_code =
            self.key_type
                .serialise_code(&String::from("kvp"), 1, &String::from("*key"));
        let value_serialise_code =
            self.value_type
                .serialise_code(&String::from("kvp"), 2, &String::from("value"));
        quote!{
            {
                for (key, value) in #value_name.iter() {
                    let kvp = ffi::Schema_AddObject(#object_name, #field_id);
                    #key_serialise_code;
                    #value_serialise_code;
                }
            }
        }
    }

    fn got_all_types(&mut self, all_types: &HashMap<String, Rc<RefCell<UserType>>>) {
        self.key_type.got_all_types(all_types);
        self.value_type.got_all_types(all_types);
    }
}
