//! Utility functions to help with generating bindings for the [Workspace] API

use std::collections::VecDeque;

use rustc_hash::FxHashSet;
use schemars::{
    JsonSchema,
    r#gen::{SchemaGenerator, SchemaSettings},
    schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec},
};
use serde_json::Value;

use crate::{WorkspaceError, workspace::*};

/// Manages a queue of type definitions that need to be generated
#[derive(Default)]
pub struct ModuleQueue<'a> {
    /// Set of type names that have already been emitted
    visited: FxHashSet<&'a str>,
    /// Queue of type names and definitions that need to be generated
    queue: VecDeque<(&'a str, &'a SchemaObject)>,
}

impl<'a> ModuleQueue<'a> {
    /// Add a type definition to the queue if it hasn't been emitted already
    fn push_back(&mut self, item: (&'a str, &'a SchemaObject)) {
        if self.visited.insert(item.0) {
            self.queue.push_back(item);
        }
    }

    /// Pull a type name and definition from the queue
    fn pop_front(&mut self) -> Option<(&'a str, &'a SchemaObject)> {
        self.queue.pop_front()
    }

    pub fn visited(&self) -> &FxHashSet<&'a str> {
        &self.visited
    }
}

/// Generate a TS type string from the `instance_type` of a [SchemaObject]
fn instance_type<'a>(
    queue: &mut ModuleQueue<'a>,
    root_schema: &'a RootSchema,
    schema: &'a SchemaObject,
    ty: InstanceType,
) -> String {
    match ty {
        // If the instance type is an object, generate a TS object type with the corresponding properties
        InstanceType::Object => {
            let object = schema.object.as_deref().unwrap();
            let mut members = Vec::new();
            for (property, prop_schema) in &object.properties {
                let (ts_type, optional, description) = schema_type(queue, root_schema, prop_schema);
                assert!(!optional, "optional nested types are not supported");

                let mut member = String::new();
                if let Some(description) = description {
                    member.push_str(&format!("/**\n\t* {description} \n\t */\n"));
                }
                member.push_str(&format!("{property}: {ts_type}"));
                members.push(member);
            }
            format!("{{ {} }}", members.join("; "))
        }
        // If the instance type is an array, generate a TS array type with the corresponding item type
        InstanceType::Array => {
            let array = schema.array.as_deref().unwrap();
            let items = array.items.as_ref().unwrap();
            match items {
                SingleOrVec::Single(schema) => {
                    let (ts_type, optional, _) = schema_type(queue, root_schema, schema);
                    assert!(!optional, "optional nested types are not supported");
                    format!("{ts_type}[]")
                }
                SingleOrVec::Vec(items) => {
                    let elements: Vec<String> = items
                        .iter()
                        .map(|schema| {
                            let (ts_type, optional, _) = schema_type(queue, root_schema, schema);
                            assert!(!optional, "optional nested types are not supported");
                            ts_type
                        })
                        .collect();
                    format!("[{}]", elements.join(", "))
                }
            }
        }

        // Map native types to the corresponding TS type
        InstanceType::Null => "null".to_string(),
        InstanceType::Boolean => "boolean".to_string(),
        InstanceType::String => "string".to_string(),
        InstanceType::Number | InstanceType::Integer => "number".to_string(),
    }
}

/// Generate a literal TS type string from a `serde_json` [Value]
fn value_type(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Number(value) => format!("{}", value.as_f64().unwrap()),
        Value::String(value) => format!("\"{value}\""),
        Value::Array(_) => unimplemented!(),
        Value::Object(_) => unimplemented!(),
    }
}

/// Generate a union type string from a list of type strings,
/// flattening any nested union types (types containing "\n\t| ")
fn make_union_type(items: impl IntoIterator<Item = String>) -> String {
    let mut result = Vec::new();

    for item in items {
        // Flatten nested union types (multi-line format)
        if item.contains("\n\t| ") {
            for part in item.split("\n\t| ") {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    result.push(trimmed.to_string());
                }
            }
        } else {
            let trimmed = item.trim();
            if !trimmed.is_empty() {
                result.push(trimmed.to_string());
            }
        }
    }

    // Format as multi-line union with leading pipe on each line for readability
    if result.len() > 1 {
        format!("\n\t| {}", result.join("\n\t| "))
    } else {
        result.into_iter().next().unwrap_or_default()
    }
}

/// Generate a TS type string from a [SchemaObject], returning the generated
/// TypeScript type along with a boolean flag indicating whether the type is
/// considered "optional" in the schema
fn schema_object_type<'a>(
    queue: &mut ModuleQueue<'a>,
    root_schema: &'a RootSchema,
    schema: &'a SchemaObject,
) -> (String, bool, Option<&'a String>) {
    // Start by detecting enum types by inspecting the `enum_values` field, if
    // the field is set return a union type generated from the literal enum values
    let description = schema
        .metadata
        .as_ref()
        .and_then(|s| s.description.as_ref());
    let ts_type = schema
        .enum_values
        .as_deref()
        .map(|enum_values| make_union_type(enum_values.iter().map(value_type)))
        // If the type isn't an enum, inspect its `instance_type` field, if the
        // field is set return a type annotation for the corresponding type
        .or_else(|| {
            Some(match schema.instance_type.as_ref()? {
                SingleOrVec::Single(ty) => instance_type(queue, root_schema, schema, **ty),
                SingleOrVec::Vec(types) => make_union_type(
                    types
                        .iter()
                        .map(|ty| instance_type(queue, root_schema, schema, *ty)),
                ),
            })
        })
        // Otherwise inspect the `reference` field of the schema, if its set return
        // a TS reference type and add the corresponding type to the queue
        .or_else(|| {
            let reference = schema.reference.as_deref()?;
            let key = reference.trim_start_matches("#/components/schemas/");
            match root_schema.definitions.get(key) {
                Some(Schema::Bool(_)) => unimplemented!(),
                Some(Schema::Object(schema)) => queue.push_back((key, schema)),
                None => panic!("definition for type {key:?} not found"),
            }

            Some(key.to_string())
        })
        // Finally try to inspect the subschemas for this type
        .or_else(|| {
            let subschemas = schema.subschemas.as_deref()?;
            // First try to inspect the `all_of` list of subschemas, if it's
            // set generate an intersection type from it
            subschemas
                .all_of
                .as_deref()
                .map(|all_of| {
                    let parts: Vec<String> = all_of
                        .iter()
                        .map(|ty| {
                            let (ts_type, optional, _) = schema_type(queue, root_schema, ty);
                            assert!(!optional, "optional nested types are not supported");
                            ts_type
                        })
                        .collect();
                    parts.join(" & ")
                })
                // Otherwise try to inspect the `any_of` list of subschemas, and
                // generate the corresponding union type for it
                .or_else(|| {
                    let any_of = subschemas
                        .any_of
                        .as_deref()
                        .or(subschemas.one_of.as_deref())?;

                    Some(make_union_type(any_of.iter().map(|ty| {
                        let (ts_type, optional, _) = schema_type(queue, root_schema, ty);
                        assert!(!optional, "optional nested types are not supported");
                        ts_type
                    })))
                })
        })
        .unwrap_or_else(|| {
            // this is temporary workaround to fix the `options` field, which is not used at the moment
            "any".to_string()
        });

    // Types are considered "optional" in the serialization protocol if they
    // have the `nullable` OpenAPI extension property, or if they have a default value
    let is_nullable = matches!(schema.extensions.get("nullable"), Some(Value::Bool(true)));
    let has_defaults = schema
        .metadata
        .as_ref()
        .is_some_and(|metadata| metadata.default.is_some());

    (ts_type, is_nullable || has_defaults, description)
}

/// Generate a TS type string from a [Schema], returning the generated type
/// along with a boolean flag indicating whether the type is considered
/// "optional" in the schema
fn schema_type<'a>(
    queue: &mut ModuleQueue<'a>,
    root_schema: &'a RootSchema,
    schema: &'a Schema,
) -> (String, bool, Option<&'a String>) {
    match schema {
        // Types defined as `true` in the schema always pass validation,
        // map them to the `any` type
        Schema::Bool(true) => ("any".to_string(), true, None),
        // Types defined as `false` in the schema never pass validation,
        // map them to the `never` type
        Schema::Bool(false) => ("never".to_string(), false, None),
        Schema::Object(schema_object) => schema_object_type(queue, root_schema, schema_object),
    }
}

/// Generate and emit all the types defined in `root_schema` into the `module`
pub fn generate_type<'a>(
    module: &mut Vec<(String, Option<&'a String>)>,
    queue: &mut ModuleQueue<'a>,
    root_schema: &'a RootSchema,
) -> String {
    // Read the root type of the schema and push it to the queue
    let root_name = root_schema
        .schema
        .metadata
        .as_deref()
        .and_then(|metadata| metadata.title.as_deref())
        .unwrap();

    match root_name {
        "Null" => return "void".to_string(),
        "Boolean" => return "boolean".to_string(),
        "String" => return "string".to_string(),
        _ => {}
    }

    queue.push_back((root_name, &root_schema.schema));

    while let Some((name, schema)) = queue.pop_front() {
        // Detect if the type being emitted is an object, emit it as an
        // interface definition if that's the case
        let is_interface = schema.instance_type.as_ref().map_or_else(
            || schema.object.is_some(),
            |instance_type| {
                if let SingleOrVec::Single(instance_type) = instance_type {
                    matches!(**instance_type, InstanceType::Object)
                } else {
                    false
                }
            },
        );

        if is_interface {
            let mut members = Vec::new();

            // Create a property signature member in the interface for each
            // property of the corresponding schema object
            let object = schema.object.as_deref().unwrap();
            for (property, prop_schema) in &object.properties {
                let (ts_type, optional, description) = schema_type(queue, root_schema, prop_schema);

                let mut member = String::new();
                if let Some(description) = description {
                    member.push_str(&format!("\t/**\n\t * {description}\n\t */\n"));
                }
                let opt = if optional { "?" } else { "" };
                member.push_str(&format!("\t{property}{opt}: {ts_type};"));
                members.push(member);
            }

            let description = schema
                .metadata
                .as_ref()
                .and_then(|s| s.description.as_ref());
            let decl = format!("interface {name} {{\n{}\n}}", members.join("\n"));
            module.push((decl, description));
        } else {
            // If the schema for this type is not an object, emit it as a type alias
            let (ts_type, optional, description) = schema_object_type(queue, root_schema, schema);

            assert!(!optional, "optional nested types are not supported");

            // For multi-line union types, don't add space after =
            let decl = if ts_type.starts_with('\n') {
                format!("type {name} ={ts_type};")
            } else {
                format!("type {name} = {ts_type};")
            };
            module.push((decl, description));
        }
    }

    root_name.to_string()
}

/// Signature metadata for a [Workspace] method
pub struct WorkspaceMethod {
    /// Name of the method
    pub name: &'static str,
    /// Schema for the parameters object of the method
    pub params: RootSchema,
    /// Schema for the result object of the method
    pub result: RootSchema,
}

impl WorkspaceMethod {
    /// Construct a [WorkspaceMethod] from a name, a parameter type and a result type
    fn of<P, R>(name: &'static str) -> Self
    where
        P: JsonSchema,
        R: JsonSchema,
    {
        let params = SchemaGenerator::from(SchemaSettings::openapi3()).root_schema_for::<P>();
        let result = SchemaGenerator::from(SchemaSettings::openapi3()).root_schema_for::<R>();
        Self {
            name,
            params,
            result,
        }
    }

    /// Construct a [WorkspaceMethod] from a name and a function pointer
    fn from_method<T, P, R>(
        name: &'static str,
        _func: fn(T, P) -> Result<R, WorkspaceError>,
    ) -> Self
    where
        P: JsonSchema,
        R: JsonSchema,
    {
        Self::of::<P, R>(name)
    }
}

/// Helper macro for generated an OpenAPI schema for a type implementing JsonSchema
macro_rules! workspace_method {
    ($name:ident) => {
        WorkspaceMethod::from_method(stringify!($name), <dyn Workspace>::$name)
    };
}

/// Returns a list of signature for all the methods in the [Workspace] trait
pub fn methods() -> [WorkspaceMethod; 9] {
    [
        workspace_method!(is_path_ignored),
        workspace_method!(register_project_folder),
        workspace_method!(get_file_content),
        workspace_method!(pull_file_diagnostics),
        workspace_method!(get_completions),
        workspace_method!(update_settings),
        workspace_method!(open_file),
        workspace_method!(change_file),
        workspace_method!(close_file),
    ]
}
