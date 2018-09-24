var fs = require("fs");
var path = require("path");
var mkdirp = require('mkdirp');

var generalModTemplate = fs.readFileSync(__dirname + "/templates/general_mod.rs", "UTF-8");
var typeTemplate = fs.readFileSync(__dirname + "/templates/type.rs", "UTF-8");
var componentTemplate = fs.readFileSync(__dirname + "/templates/component.rs", "UTF-8");
var componentFileHeaderTemplate = fs.readFileSync(__dirname + "/templates/component_file_header.rs", "UTF-8");
var commandTemplate = fs.readFileSync(__dirname + "/templates/command.rs", "UTF-8");

var builtInMap = {
	double: {
		type: "f64",
		method: "Double"
	},
	float: {
		type: "f32",
		method: "Float"
	},
	uint32: {
		type: "u32",
		method: "Uint32"
	},
	string: {
		type: "String",
		method: "String"
	},
	EntityId: {
		type: "EntityId",
		method: "EntityId"
	}
}

function snakeToTitleCase(snakeCase) {
	return snakeCase.split('_').map(function (item) {
        return item.charAt(0).toUpperCase() + item.substring(1);
    }).join('');
}


var jsonDirectory = process.argv[2];
var outputDirectory = process.argv[3];


function findJsonFiles(dir) {
    var results = [];
    var list = fs.readdirSync(dir);
    list.forEach(function(file) {
        file = dir + '/' + file;
        var stat = fs.statSync(file);
        if (stat && stat.isDirectory()) { 
            /* Recurse into a subdirectory */
            results = results.concat(findJsonFiles(file));
        } else { 
            /* Is a file */
            if(file.endsWith(".json")){
            	results.push(file);
            }
        }
    });
    return results;
}

function filePathForPackage(package) {
	return path.join(outputDirectory, package.replace(/\./g, '/'));
}

function ensurePackagePathExists(package){
	mkdirp.sync(filePathForPackage(package));
}

class OutputTree {
	constructor(allTypes, allComponents) {
		this.tree = {
			file: this.getGeneralFile(allTypes, allComponents),
			children: {}
		};
	}

	getGeneralFile(allTypes, allComponents){
		var uniqueIndexMatchCode = "";
		var dataEnumDefCode = "";
		var updateEnumDefCode = "";
		var dataDeserialise = "";
		var dataSerialise = "";
		var dataApplyUpdate = "";
		var updateDeserialise = "";
		var updateSerialise = "";
		var dynamicHandlerCode = "";
		var componentRequestDeserialiseMatch = "";
		var componentResponseDeserialiseMatch = "";

		var uniqueIndex = 0;
		for(var key in allComponents){
			var component = allComponents[key];

			var enumName = component.enumName;

			uniqueIndexMatchCode += component.id + " => Some(" + uniqueIndex + "),\n";
			uniqueIndex++;

			dataEnumDefCode += enumName + "(" + component.languageQualifiedName + "Data),\n";
			updateEnumDefCode += enumName + "(" + component.languageQualifiedName + "Update),\n";
			dataDeserialise += component.id + " => Some(" + component.languageQualifiedName + "Data::deserialise_data(data)),\n";
			dataSerialise += "&ComponentData::" + enumName + "(ref data) => data.serialise_data(),\n";
			dataApplyUpdate += "&mut ComponentData::" + enumName + "(ref mut data) => {\n\
				if let &ComponentUpdate::" + enumName + "(ref update) = update {\n\
					data.apply_update(&update);\n\
				}\n\
			}\n";
			updateDeserialise += component.id + " => Some(" + component.languageQualifiedName + "Update::deserialise_update(update)),\n";
			updateSerialise += "&ComponentUpdate::" + enumName + "(ref update) => update.serialise_update(),\n";
			dynamicHandlerCode += "handler.register_component::<" + component.languageQualifiedName + ">();\n";

			for(var i in component.commands){
				var command = component.commands[i];
				componentRequestDeserialiseMatch += 
				`(${component.id}, ${command.commandIndex}) => 
					Some(Box::new(${command.requestType.languageTypeName}::deserialise_request(request))),\n`;
				componentResponseDeserialiseMatch += 
				`(${component.id}, ${command.commandIndex}) => 
					Some(Box::new(${command.responseType.languageTypeName}::deserialise_response(response))),\n`;
			}
		}

		var numberOfComponents = uniqueIndex;

		return generalModTemplate
			.replace(/{{UNIQUE_INDEX_MATCH}}/g, uniqueIndexMatchCode)
			.replace(/{{NUMBER_OF_COMPONENTS}}/g, numberOfComponents)
			.replace(/{{DATA_ENUM_DEF}}/g, dataEnumDefCode)
			.replace(/{{UPDATE_ENUM_DEF}}/g, updateEnumDefCode)
			.replace(/{{DATA_DESERIALISE}}/g, dataDeserialise)
			.replace(/{{DATA_SERIALISE}}/g, dataSerialise)
			.replace(/{{DATA_APPLY_UPDATE}}/g, dataApplyUpdate)
			.replace(/{{UPDATE_DESERIALISE}}/g, updateDeserialise)
			.replace(/{{UPDATE_SERIALISE}}/g, updateSerialise)
			.replace(/{{DYNAMIC_HANDLER_CODE}}/g, dynamicHandlerCode)
			.replace(/{{COMMAND_REQUEST_DESERIALISE_MATCH}}/g, componentRequestDeserialiseMatch)
			.replace(/{{COMMAND_RESPONSE_DESERIALISE_MATCH}}/g, componentResponseDeserialiseMatch)
	}

	getPackageNode(packageName){
		var packageNames = packageName.split(".");
		var node = this.tree;
		for(var i in packageNames){
			var child = node.children[packageNames[i]];
			if(!child){
				// Not seen this package yet
				child = {
					file: componentFileHeaderTemplate + "\n\n",
					children: {}
				}
				node.children[packageNames[i]] = child;
				node.file += "pub mod " + packageNames[i] + ";\n";
			}
			node = child;
		}
		return node;
	}

	flushToFiles() {
		this.flushNodeToFiles(this.tree, "");
	}

	flushNodeToFiles(node, packageName) {
		var filePath = path.join(filePathForPackage(packageName.substring(1)), "mod.rs");
		fs.writeFileSync(filePath, node.file);

		for(var child in node.children){
			this.flushNodeToFiles(node.children[child], packageName + "." + child);
		}
	}

	writeType(type) {
		var node = this.getPackageNode(type.packageName);

		var code = type.generateCode();
		node.file += "\n\n" + code;
	}

	writeComponent(allTypes, component) {
		var node = this.getPackageNode(component.packageName);

		var code = component.generateCode(allTypes);
		node.file += "\n\n" + code;
	}
}

class SchemaType {
	constructor(object){
		if(object.singularType){
			if(object.singularType.builtInType){
				this.isBuiltIn = true;
				var type = builtInMap[object.singularType.builtInType];
				this.languageTypeName = type.type;
				this.typeMethodGroup = type.method;
			}else{
				this.isBuiltIn = false;
				this.languageTypeName = "generated::" + object.singularType.userType.split(".").join("::");
			}
		}else if(object.optionType){
			this.isOption = true;
			this.isBuiltIn = false;
			this.valueType = new SchemaType({
				singularType: object.optionType.valueType
			});
			this.languageTypeName = "Option<" + this.valueType.languageTypeName + ">";
		}else if(object.listType){
			this.isList = true;
			this.isBuiltIn = false;
			this.valueType = new SchemaType({
				singularType: object.listType.valueType
			});
			this.languageTypeName = "Vec<" + this.valueType.languageTypeName + ">";
		}else if(object.mapType){
			// It's a map
			this.isMap = true;

			this.keyType = new SchemaType({
				singularType: object.mapType.keyType
			});
			this.valueType = new SchemaType({
				singularType: object.mapType.valueType
			});

			this.isBuiltIn = false;
			this.languageTypeName = "HashMap<" + this.keyType.languageTypeName + ", " + this.valueType.languageTypeName + ">";
		}
	}

	getDeserialiseCode(objectName, fieldId, index = "0") {
		if(this.isBuiltIn){
			return "ffi::Schema_Index" + this.typeMethodGroup + "(" + objectName + ", " + fieldId + ", " + index + ")";
		}else{
			if(this.isOption){
				return "{\n\
							if " + this.getCountCode(objectName, fieldId) + " > 0 {\n\
								Some(" + this.valueType.getDeserialiseCode(objectName, fieldId) + ")\n\
							} else { None }\n\
						}";
			}if(this.isList){
				return "{\n\
							let count = " + this.getCountCode(objectName, fieldId) + ";\n\
							let mut list: " + this.languageTypeName + " = Vec::with_capacity(count as usize);\n\
							for i in 0..count {\n\
								list.push(" + this.valueType.getDeserialiseCode(objectName, fieldId, "i") + ");\n\
							}\n\
							list\n\
						}";
			}if(this.isMap){
				return "{\n\
							let count = " + this.getCountCode(objectName, fieldId) + ";\n\
							let mut map: " + this.languageTypeName + " = HashMap::with_capacity(count as usize);\n\
							for i in 0..count {\n\
								let kvp = ffi::Schema_IndexObject(" + objectName + ", " + fieldId + ", i);\n\
								map.insert(" + this.keyType.getDeserialiseCode("kvp", 1) + ", " + this.valueType.getDeserialiseCode("kvp", 2) + ");\n\
							}\n\
							map\n\
						}";
			}else{
				return this.languageTypeName + "::deserialise(ffi::Schema_IndexObject(" + objectName + ", " + fieldId + ", " + index + "))";
			}
		}
	}

	getSerialiseCode(objectName, fieldId, valueName) {
		if(this.isBuiltIn){
			if(this.typeMethodGroup == "String"){
				valueName = "&" + valueName;
			}
			return "ffi::Schema_Add" + this.typeMethodGroup + "(" + objectName + ", " + fieldId + ", " + valueName + ")";
		}else{
			if(this.isOption){
				return "{\n\
							if let Some(ref value) = " + valueName + " {\n\
								" + this.valueType.getSerialiseCode(objectName, fieldId, "*value") + "\n\
							}\n\
						}";
			}if(this.isList){
				return "{\n\
							for ref value in " + valueName + ".iter() {\n\
								" + this.valueType.getSerialiseCode(objectName, fieldId, "value") + ";\n\
							}\n\
						}";
			}if(this.isMap){
				return "{\n\
							for (key, value) in &" + valueName + " {\n\
								let kvp = ffi::Schema_AddObject(" + objectName + ", " + fieldId + ");\n\
								" + this.keyType.getSerialiseCode("kvp", 1, "*key") + ";\n\
								" + this.valueType.getSerialiseCode("kvp", 2, "value") + ";\n\
							}\n\
						}";
			}
			return valueName + ".serialise(ffi::Schema_AddObject(" + objectName + ", " + fieldId + "))";
		}
	}

	getCountCode(objectName, fieldId) {
		if(this.isBuiltIn){
			return "ffi::Schema_Get" + this.typeMethodGroup + "Count(" + objectName + ", " + fieldId + ")";
		}else{
			return "ffi::Schema_GetObjectCount(" + objectName + ", " + fieldId + ")";
		}
	}
}

class Event {
	constructor(object){
		this.name = object.name;
		this.eventIndex = object.eventIndex;

		this.type = new SchemaType({
			singularType: {
				userType: object.type.userType
			}
		});

		this.eventListType = new SchemaType({
			listType: {
				valueType: {
					userType: object.type.userType
				}
			}
		});
	}

	getDefinitionCode() {
		return `pub ${this.name}: Event<${this.type.languageTypeName}>`;
	}

	getUpdateDefinitionCode() {
		return `${this.name}: Vec<${this.type.languageTypeName}>`;
	}

	getReadUpdateCode() {
		return `${this.name}: { \
					let events_obj = ffi::Schema_GetComponentUpdateEvents(update);
					${this.eventListType.getDeserialiseCode("events_obj", this.eventIndex)}
				}`
	}

	getInitialCode() {
		return `${this.name}: Event::new()`;
	}

	getContainsEventsCode(){
		return `self.${this.name}.len() > 0`;
	}

	getClearEventsCode(){
		return `self.${this.name}.clear()`;
	}

	getSerialiseEventUpdateCode() {
		return `${this.eventListType.getSerialiseCode("events", this.eventIndex, `self.${this.name}.get_staged_events()`)}\n
				self.${this.name}.clear_staged_events();`;
	}

	getApplyUpdatesEventsCode() {
		return `for value in update.${this.name}.iter() {\n
					self.${this.name}.add_event(value.clone());
				}`;
	}

	getSnapshotToDataFieldCode() {
		return `${this.name}: Event::new()`;
	}
}

class Command {
	constructor(object, component) {
		this.name = object.name;
		this.component = component;
		this.commandIndex = object.commandIndex;

		this.requestType = new SchemaType({
			singularType: {
				userType: object.requestType.userType
			}
		});

		this.responseType = new SchemaType({
			singularType: {
				userType: object.responseType.userType
			}
		});

		this.commandStructName = component.name + snakeToTitleCase(this.name);
	}

	getCode() {
		return commandTemplate
			.replace(/{{COMMAND_STRUCT_NAME}}/g, this.commandStructName)
			.replace(/{{REQUEST_TYPE}}/g, this.requestType.languageTypeName)
			.replace(/{{RESPONSE_TYPE}}/g, this.responseType.languageTypeName)
			.replace(/{{COMPONENT_NAME}}/g, this.component.name)
			.replace(/{{COMMAND_INDEX}}/g, this.commandIndex);
	}

	getCommandGetterCode() {
		return `pub fn ${this.name}() -> ${this.commandStructName} {
					${this.commandStructName} \{\}
				}`;
	}
}

class Field {
	constructor(object) {
		this.name = object.name;
		this.fieldId = object.number;
		this.isDataProperty = false;

		this.type = new SchemaType(object);

		// if(object.singularType){
		// 	this.type = new SchemaType(object.singularType);
		// }else if(object.listType){
		// 	this.type = new SchemaType(object.listType.valueType);
		// 	this.type.languageTypeName = "Vec<" + this.type.languageTypeName + ">";

		// 	// var typeInfo = this.getTypeInfo(object.listType.valueType);

		// 	// this.isBuiltIn = false;
		// 	// this.languageTypeName = "Vec<" + typeInfo.type + ">";

		// }else if(object.mapType){
		// 	this.isMap = true;
		// 	this.type = new SchemaType(object.mapType);
		// 	// this.valueType = new Type();
		// 	// this.type = {
		// 	// 	languageTypeName: "HashMap<" + this.keyType.languageTypeName + ", " + this.valueType.languageTypeName + ">"
		// 	// };

		// 	// this.isBuiltIn = false;
		// 	// this.type = "HashMap<" + this.keyTypeInfo.languageTypeName + ", " + this.valueTypeInfo.languageTypeName + ">";
		// }
	}

	// getTypeInfo(typeObject) {
	// 	if(typeObject.builtInType){
	// 		this.isBuiltIn = true;
	// 		var type = builtInMap[typeObject.builtInType];
	// 		return {
	// 			isBuiltIn: true,
	// 			languageTypeName: type.type,
	// 			typeMethodGroup: type.method
	// 		}
	// 	}else{
	// 		return {
	// 			isBuiltIn: false,
	// 			languageTypeName: "generated::" + typeObject.userType.split(".").join("::")
	// 		}
	// 	}
	// }

	getDefinitionCode(wrapInProperty = this.isDataProperty) {
		var typeName = this.type.languageTypeName;
		if(wrapInProperty){
			typeName = `Property<${typeName}>`;
		}
		return `pub ${this.name}: ${typeName}`;
	}

	getUpdateDefinitionCode() {
		return "pub " + this.name + ": Option<" + this.type.languageTypeName + ">";
	}

	getApplyUpdatesFieldsCode() {
		var valueCode = "value.clone()";
		if(this.isDataProperty){
			valueCode = `Property::new(value.clone())`;
		}
		return `if let Some(ref value) = update.${this.name} {\n\
			self.${this.name} = ${valueCode};\n\
		}`;
	}

	getReadDataCode() {
		if(this.isDataProperty){
			return `${this.name}: Property::new(${this.type.getDeserialiseCode("object", this.fieldId)})`;
		}else{
			return `${this.name}: ${this.type.getDeserialiseCode("object", this.fieldId)}`;
		}
	}

	getWriteDataCode() {
		var valueName = `self.${this.name}`;
		if(this.isDataProperty){
			valueName += ".deref()";
			if(this.type.isBuiltIn || this.type.isMap || this.type.isList || this.type.isOption){
				valueName = "*" + valueName;
			}
		}
		return this.type.getSerialiseCode("object", this.fieldId, valueName);
	}

	getReadUpdateCode() {
		return this.name + ": {\n\
			    	if " + this.type.getCountCode("fields", this.fieldId) + " > 0 {\n\
						Some(" + this.type.getDeserialiseCode("fields", this.fieldId) + ")\n\
					}else{ None }\n\
				}";
	}

	getWriteUpdateCode() {
		var valueName = "value";
		if(this.type.isBuiltIn || this.type.isMap || this.type.isList || this.type.isOption){
			valueName = "*" + valueName;
		}
		return "if let Some(ref value) = self." + this.name + " {\n\
				" + this.type.getSerialiseCode("fields", this.fieldId, valueName) + ";\n\
			}"
	}

	// getPropertyGetter() {
	// 	return `pub fn ${this.name}(&self) -> &${this.type.languageTypeName} { &self.${this.name} }`
	// }

	// getPropertySetter(fieldIndex) {
	// 	return `pub fn ${this.name}_mut(&mut self) -> &mut ${this.type.languageTypeName} { \
	// 			self.mark_as_dirty(${fieldIndex});
	// 			&mut self.${this.name} \
	// 		}`
	// }

	getDirtyUpdateField() {
		var valueName = `self.${this.name}`;
		if(this.isDataProperty){
			valueName += ".deref()";
			if(this.type.isBuiltIn || this.type.isMap || this.type.isList || this.type.isOption){
				valueName = "*" + valueName;
			}
		}
		return `if self.${this.name}.get_and_clear_dirty_bit() { \
					${this.type.getSerialiseCode("fields", this.fieldId, valueName)} \
				}`
	}

	getDirtyPropertiesCode(){
		return `self.${this.name}.get_dirty_bit() ||`;
	}

	getSnapshotToDataFieldCode(){
		return `${this.name}: self.${this.name}.into()`;
	}
}

class Type {
	constructor(packageName, object) {
		this.packageName = packageName;
		this.name = object.name;
		this.qualifiedName = object.qualifiedName;
		this.isDataType = false;

		this.fields = [];
		for(var i in object.fieldDefinitions){
			this.fields.push(new Field(object.fieldDefinitions[i]));
		}
	}

	generateCode(){
		var definitionCode = "";
		var readFieldsCode = "";
		var writeFieldsCode = "";

		for(var i in this.fields){
			var field = this.fields[i];

			definitionCode += field.getDefinitionCode() + ",\n";
			readFieldsCode += field.getReadDataCode() + ",\n";
			writeFieldsCode += field.getWriteDataCode() + ";\n";
		}
		if(this.isDataType){
			// definitionCode += `dirty_fields: [bool; ${this.fields.length}],`;
			definitionCode += `is_dirty: bool,\n`;			
			// readFieldsCode += `dirty_fields: [false; ${this.fields.length}],`;
			readFieldsCode += `is_dirty: false,\n`;
		}
		for(var i in this.events){
			var event = this.events[i];

			definitionCode += event.getDefinitionCode() + ",\n";
			readFieldsCode += event.getInitialCode() + ",\n";
		}

		return typeTemplate
			.replace(/{{NAME}}/g, this.name)
			.replace(/{{DEF_FIELDS}}/g, definitionCode)
			.replace(/{{READ_FIELDS}}/g, readFieldsCode)
			.replace(/{{WRITE_FIELDS}}/g, writeFieldsCode)
	}
}

class Component {
	constructor(packageName, object) {
		this.packageName = packageName;
		this.name = object.name;
		this.id = object.id;
		this.enumName = "Component" + this.id;
		this.qualifiedName = object.qualifiedName;
		this.languageQualifiedName = "generated::" + this.qualifiedName.split(".").join("::");

		this.events = [];
		for(let i in object.eventDefinitions){
			this.events.push(new Event(object.eventDefinitions[i]));
		}

		this.commands = [];
		for(let i in object.commandDefinitions){
			this.commands.push(new Command(object.commandDefinitions[i], this));
		}

		this.dataTypeName = object.dataDefinition.userType;
	}

	parsingFinished(allTypes, allComponents) {
		var dataType = allTypes[this.dataTypeName];
		dataType.isDataType = true;
		for(i in dataType.fields){
			dataType.fields[i].isDataProperty = true;
		}
		dataType.events = this.events;
	}

	generateCode(allTypes){
		var dataType = allTypes[this.dataTypeName];

		var definitionCode = "";
		var updateDefinitionCode = "";
		var applyUpdateFieldsCode = "";
		var applyUpdateEventsCode = "";
		var readUpdateCode = "";
		var writeUpdateCode = "";
		// var propertyGetters = "";
		// var propertySetters = "";
		var dirtyUpdateFields = "";
		var dirtyPropertiesCode = "";
		var containsEventsCode = "";
		var clearEventsCode = "";
		var serialiseEventUpdateCode = "";
		var commandGetterCode = "";
		var commandCode = "";
		var snapshotToDataFieldsCode = "";

		for(var i in dataType.fields){
			var field = dataType.fields[i];
			definitionCode += field.getDefinitionCode(false) + ",\n";
			updateDefinitionCode += field.getUpdateDefinitionCode() + ",\n";
			applyUpdateFieldsCode += field.getApplyUpdatesFieldsCode() + "\n";
			readUpdateCode += field.getReadUpdateCode() + ",\n";
			writeUpdateCode += field.getWriteUpdateCode() + "\n";
			// propertyGetters += field.getPropertyGetter() + "\n";
			// propertySetters += field.getPropertySetter(i) + "\n";
			dirtyUpdateFields += field.getDirtyUpdateField(i) + "\n";
			dirtyPropertiesCode += field.getDirtyPropertiesCode() + "\n";
			snapshotToDataFieldsCode += field.getSnapshotToDataFieldCode() + ",\n";
		}

		for(var i in this.events){
			var event = this.events[i];
			updateDefinitionCode += event.getUpdateDefinitionCode() + ",\n";
			readUpdateCode += event.getReadUpdateCode() + ",\n";
			containsEventsCode += event.getContainsEventsCode() + " || ";
			clearEventsCode += event.getClearEventsCode() + ";";
			serialiseEventUpdateCode += event.getSerialiseEventUpdateCode();
			applyUpdateEventsCode += event.getApplyUpdatesEventsCode() + "\n";
			snapshotToDataFieldsCode += event.getSnapshotToDataFieldCode() + ",\n";
		}

		for(var i in this.commands) {
			var command = this.commands[i];
			commandGetterCode += command.getCommandGetterCode();
			commandCode += command.getCode();
		}

		return componentTemplate
			.replace(/{{NAME}}/g, this.name)
			.replace(/{{ENUM_NAME}}/g, this.enumName)
			.replace(/{{COMPONENT_ID}}/g, this.id)
			.replace(/{{DEF_FIELDS}}/g, definitionCode)
			.replace(/{{DEF_UPDATE_FIELDS}}/g, updateDefinitionCode)
			.replace(/{{APPLY_UPDATE_FIELDS}}/g, applyUpdateFieldsCode)
			.replace(/{{APPLY_UPDATE_EVENTS}}/g, applyUpdateEventsCode)
			.replace(/{{READ_UPDATE_FIELDS}}/g, readUpdateCode)
			.replace(/{{WRITE_UPDATE_FIELDS}}/g, writeUpdateCode)
			// .replace(/{{PROP_GETTERS}}/g, propertyGetters)
			// .replace(/{{PROP_SETTERS}}/g, propertySetters)
			.replace(/{{DIRTY_UPDATE_FIELDS}}/g, dirtyUpdateFields)
			.replace(/{{DIRTY_PROPERTIES_CODE}}/g, dirtyPropertiesCode)
			.replace(/{{CONTAINS_EVENTS}}/g, containsEventsCode)
			.replace(/{{CLEAR_EVENTS}}/g, clearEventsCode)
			.replace(/{{SERIALISE_UPDATE_EVENTS}}/g, serialiseEventUpdateCode)
			.replace(/{{COMMAND_GETTERS}}/g, commandGetterCode)
			.replace(/{{SNAPSHOT_TO_DATA_FIELDS}}/g, snapshotToDataFieldsCode) + "\n\n" + commandCode;
	}
}

function parseSchemaJson(types, components, jsonFile){
	var schema = JSON.parse(fs.readFileSync(jsonFile, "UTF-8"));
	var packageName = schema.package;

	ensurePackagePathExists(packageName);

	for(var i in schema.typeDefinitions){
		var type = new Type(packageName, schema.typeDefinitions[i]);
		types[type.qualifiedName] = type;
	}

	for(var i in schema.componentDefinitions){
		var component = new Component(packageName, schema.componentDefinitions[i]);
		components[component.qualifiedName] = component;

		console.log("Parsing " + component.name);
	}
}

var jsonFiles = findJsonFiles(jsonDirectory);

var allTypes = {};
var allComponents = {};

for(var i in jsonFiles){
	console.log("Parsing " + jsonFiles[i]);
	parseSchemaJson(allTypes, allComponents, jsonFiles[i]);
}

for(var key in allComponents){
	allComponents[key].parsingFinished(allTypes, allComponents);
}

var outputTree = new OutputTree(allTypes, allComponents);

for(var typeName in allTypes){
	outputTree.writeType(allTypes[typeName]);
}

for(var componentName in allComponents){
	outputTree.writeComponent(allTypes, allComponents[componentName]);
}

// console.log(JSON.stringify(outputTree.tree, null, 4));

// console.log(outputTree.tree.children.improbable.children.ship.file);

outputTree.flushToFiles();
