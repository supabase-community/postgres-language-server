use crate::update;
use crate::{project_root, Mode, Result};
use pgls_catalog::Snapshot;
use pgls_configuration::PartialConfiguration;
use pgls_workspace::workspace_types::{generate_type, ModuleQueue};
use schemars::r#gen::{SchemaGenerator, SchemaSettings};
use schemars::JsonSchema;

/// Generates the TypeScript types of the WASM package: the database schema and the
/// configuration.
pub fn generate_schema_types(mode: Mode) -> Result<()> {
    let wasm_src = project_root().join("packages/@postgres-language-server/wasm/src");
    update(
        &wasm_src.join("schema-cache.d.ts"),
        &type_declarations::<Snapshot>(),
        &mode,
    )?;
    update(
        &wasm_src.join("configuration.d.ts"),
        &type_declarations::<PartialConfiguration>(),
        &mode,
    )?;

    Ok(())
}

/// The TypeScript declarations of `T` and of the types it uses.
fn type_declarations<T: JsonSchema>() -> String {
    let mut declarations: Vec<(String, Option<&String>)> = Vec::new();
    let mut queue = ModuleQueue::default();
    let schema = SchemaGenerator::from(SchemaSettings::openapi3()).root_schema_for::<T>();

    generate_type(&mut declarations, &mut queue, &schema);

    let mut output = String::new();
    output.push_str("// Generated file, do not edit by hand, see `xtask/codegen`\n");

    for (decl, description) in &declarations {
        if let Some(description) = description {
            output.push_str(&format!("/**\n * {description}\n */\n"));
        }
        output.push_str(&format!("export {decl}\n"));
    }

    output
}
