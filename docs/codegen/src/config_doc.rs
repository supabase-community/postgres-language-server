use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::{Map, Value};

pub fn generate_config_doc(docs_root: &Path) -> Result<()> {
    let schema_path = docs_root.join("schema.json");
    let output_path = docs_root.join("reference/configuration.md");

    let schema_content = fs::read_to_string(&schema_path)?;
    let schema: Value = serde_json::from_str(&schema_content)?;

    let config_doc = generate_config_markdown(&schema)?;

    // Read existing file and replace the section
    let existing_content = fs::read_to_string(&output_path)?;
    let new_content = crate::utils::replace_section(&existing_content, "CONFIG_DOC", &config_doc);

    fs::write(&output_path, new_content)?;

    println!(
        "Generated configuration documentation at {}",
        output_path.display()
    );
    Ok(())
}

fn generate_config_markdown(schema: &Value) -> Result<String> {
    let mut output = String::new();

    if let Some(description) = schema.get("description").and_then(|d| d.as_str()) {
        output.push_str(&format!("{}\n\n", description));
    }

    // Get root properties
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        // Get definitions for references
        let empty_map = Map::new();
        let definitions = schema
            .get("definitions")
            .and_then(|d| d.as_object())
            .unwrap_or(&empty_map);

        output.push_str("## Configuration Options\n\n");

        // Sort properties alphabetically, but put $schema first if it exists
        let mut sorted_props: Vec<_> = properties.iter().collect();
        sorted_props.sort_by_key(|(key, _)| {
            if *key == "$schema" {
                "00_schema" // Sort first
            } else {
                key
            }
        });

        for (key, value) in sorted_props {
            if key == "$schema" {
                continue; // Skip $schema in main docs
            }

            output.push_str(&generate_property_section(key, value, definitions, 2)?);
        }
    }

    Ok(output)
}

fn generate_property_section(
    key: &str,
    property: &Value,
    definitions: &Map<String, Value>,
    level: usize,
) -> Result<String> {
    let mut output = String::new();
    let header = "#".repeat(level);

    // Property name as header
    output.push_str(&format!("{} `{}`\n\n", header, key));

    // Add description if available
    if let Some(description) = property.get("description").and_then(|d| d.as_str()) {
        output.push_str(&format!("{}\n\n", description));
    }

    // Handle type information
    let type_info = extract_type_info(property, definitions);
    if !type_info.is_empty() {
        output.push_str(&format!("**Type:** {}\n\n", type_info));
    }

    // Handle default value
    if let Some(default) = property.get("default") {
        output.push_str(&format!(
            "**Default:** `{}`\n\n",
            format_json_value(default)
        ));
    }

    // Handle object properties (nested configuration)
    if let Some(obj_ref) = extract_ref(property) {
        if let Some(definition) = definitions.get(&obj_ref) {
            output.push_str(&generate_object_properties(
                definition,
                definitions,
                level + 1,
            )?);
        }
    } else if property.get("type").and_then(|t| t.as_str()) == Some("object") {
        output.push_str(&generate_object_properties(
            property,
            definitions,
            level + 1,
        )?);
    }

    // Handle array items
    if let Some(items) = property.get("items") {
        output.push_str(&format!("{}### Array Items\n\n", header));
        if let Some(items_ref) = extract_ref(items) {
            if let Some(definition) = definitions.get(&items_ref) {
                output.push_str(&generate_object_properties(
                    definition,
                    definitions,
                    level + 2,
                )?);
            }
        }
    }

    // Handle enum values
    if let Some(enum_values) = property.get("enum").and_then(|e| e.as_array()) {
        output.push_str(&format!("{}### Possible Values\n\n", header));
        for value in enum_values {
            output.push_str(&format!("- `{}`\n", format_json_value(value)));
        }
        output.push('\n');
    }

    // Add example if this is a top-level config section
    if level == 2 {
        output.push_str(&generate_example_section(key, property, definitions)?);
    }

    Ok(output)
}

fn generate_object_properties(
    object: &Value,
    definitions: &Map<String, Value>,
    level: usize,
) -> Result<String> {
    let mut output = String::new();

    if let Some(properties) = object.get("properties").and_then(|p| p.as_object()) {
        for (prop_key, prop_value) in properties {
            output.push_str(&generate_property_section(
                prop_key,
                prop_value,
                definitions,
                level,
            )?);
        }
    }

    Ok(output)
}

fn generate_example_section(
    key: &str,
    _property: &Value,
    _definitions: &Map<String, Value>,
) -> Result<String> {
    let mut output = String::new();

    // Generate basic examples for common config sections
    let example = match key {
        "db" => {
            r#"```jsonc
{
  "db": {
    "host": "localhost",
    "port": 5432,
    "database": "postgres",
    "user": "postgres",
    "password": "password"
  }
}
```"#
        }
        "linter" => {
            r#"```jsonc
{
  "linter": {
    "enabled": true,
    "rules": {
      "all": true
    }
  }
}
```"#
        }
        "files" => {
            r#"```jsonc
{
  "files": {
    "include": ["**/*.sql"],
    "ignore": ["**/migrations/*.sql"]
  }
}
```"#
        }
        _ => return Ok(output), // No example for unknown sections
    };

    output.push_str("### Example\n\n");
    output.push_str(example);
    output.push_str("\n\n");

    Ok(output)
}

fn extract_type_info(property: &Value, definitions: &Map<String, Value>) -> String {
    // Handle direct type
    if let Some(type_str) = property.get("type").and_then(|t| t.as_str()) {
        return type_str.to_string();
    }

    // Handle anyOf with null (optional types)
    if let Some(any_of) = property.get("anyOf").and_then(|a| a.as_array()) {
        let mut types = Vec::new();
        for item in any_of {
            if let Some(type_str) = item.get("type").and_then(|t| t.as_str()) {
                if type_str != "null" {
                    types.push(type_str.to_string());
                }
            } else if let Some(ref_str) = extract_ref(item) {
                if let Some(definition) = definitions.get(&ref_str) {
                    if let Some(desc) = definition.get("description").and_then(|d| d.as_str()) {
                        // Use the first line of description as type hint
                        types.push(desc.lines().next().unwrap_or(&ref_str).to_string());
                    } else {
                        types.push(ref_str);
                    }
                }
            }
        }
        if !types.is_empty() {
            let is_optional = any_of
                .iter()
                .any(|item| item.get("type").and_then(|t| t.as_str()) == Some("null"));
            let type_str = types.join(" | ");
            return if is_optional {
                format!("{} (optional)", type_str)
            } else {
                type_str
            };
        }
    }

    // Handle $ref
    if let Some(ref_str) = extract_ref(property) {
        return ref_str;
    }

    "unknown".to_string()
}

fn extract_ref(value: &Value) -> Option<String> {
    value
        .get("$ref")
        .and_then(|r| r.as_str())
        .and_then(|r| r.strip_prefix("#/definitions/"))
        .map(|r| r.to_string())
}

fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => {
            let items: Vec<_> = arr.iter().map(format_json_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(_) => "{ ... }".to_string(),
        Value::Null => "null".to_string(),
    }
}
