use serde_json;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct JsonCollection {
    pub typeDefinitions: Vec<TypeDefinition>,
    pub componentDefinitions: Vec<ComponentDefinition>,
}

impl JsonCollection {
    pub fn append(&mut self, mut other: JsonCollection) {
        self.typeDefinitions.append(&mut other.typeDefinitions);
        self.componentDefinitions
            .append(&mut other.componentDefinitions);
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentDefinition {
    pub id: u32,
    pub name: String,
    pub qualifiedName: String,
    pub dataDefinition: SchemaTypeDefinition,
    pub eventDefinitions: Vec<EventDefinition>,
    pub commandDefinitions: Vec<CommandDefinition>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventDefinition {
    pub name: String,

    #[serde(rename = "type")]
    pub eventType: SchemaTypeDefinition,

    pub eventIndex: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandDefinition {
    pub name: String,
    pub requestType: SchemaTypeDefinition,
    pub responseType: SchemaTypeDefinition,
    pub commandIndex: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypeDefinition {
    pub fieldDefinitions: Vec<FieldDefinition>,
    pub name: String,
    pub qualifiedName: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,
    pub number: u32,
    pub singularType: Option<SchemaTypeDefinition>,
    pub optionType: Option<OptionTypeDefinition>,
    pub listType: Option<ListTypeDefinition>,
    pub mapType: Option<MapTypeDefinition>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapTypeDefinition {
    pub keyType: SchemaTypeDefinition,
    pub valueType: SchemaTypeDefinition,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptionTypeDefinition {
    pub valueType: SchemaTypeDefinition,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListTypeDefinition {
    pub valueType: SchemaTypeDefinition,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaTypeDefinition {
    pub builtInType: Option<String>,
    pub userType: Option<String>,
}

pub fn parse_json<P: AsRef<Path>>(path: P) -> Result<JsonCollection, Box<Error>> {
    let file = File::open(path)?;
    let json = serde_json::from_reader(file)?;
    Ok(json)
}
