use crate::update;
use pgls_schema_cache::SchemaCache;
use pgls_workspace::workspace_types::{generate_type, ModuleQueue};
use schemars::r#gen::{SchemaGenerator, SchemaSettings};
use xtask::{project_root, Mode, Result};

pub fn generate_wasm_schema_types(mode: Mode) -> Result<()> {
    let output_path =
        project_root().join("packages/@postgres-language-server/wasm/src/schema-cache.d.ts");

    let mut declarations: Vec<(String, Option<&String>)> = Vec::new();
    let mut queue = ModuleQueue::default();
    let schema = SchemaGenerator::from(SchemaSettings::openapi3()).root_schema_for::<SchemaCache>();

    generate_type(&mut declarations, &mut queue, &schema);

    let mut output = String::new();
    output.push_str("// Generated file, do not edit by hand, see `xtask/codegen`\n");

    for (decl, description) in &declarations {
        if let Some(description) = description {
            output.push_str(&format!("/**\n * {description}\n */\n"));
        }
        output.push_str(&format!("export {decl}\n"));
    }

    update(&output_path, &output, &mode)?;

    Ok(())
}
