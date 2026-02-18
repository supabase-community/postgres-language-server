use crate::update;
use convert_case::{Case, Casing};
use pgls_workspace::workspace_types::{generate_type, methods, ModuleQueue};
use xtask::{project_root, Mode, Result};

pub fn generate_bindings(mode: Mode) -> Result<()> {
    generate_workspace_bindings(&mode)
}

fn generate_workspace_bindings(mode: &Mode) -> Result<()> {
    let bindings_path_postgrestools =
        project_root().join("packages/@postgrestools/backend-jsonrpc/src/workspace.ts");
    let bindings_path_postgres_language_server =
        project_root().join("packages/@postgres-language-server/backend-jsonrpc/src/workspace.ts");
    let methods = methods();

    let mut declarations: Vec<(String, Option<&String>)> = Vec::new();
    let mut workspace_members = Vec::new();
    let mut impl_members = Vec::new();
    let mut queue = ModuleQueue::default();

    for method in &methods {
        let params = generate_type(&mut declarations, &mut queue, &method.params);
        let result = generate_type(&mut declarations, &mut queue, &method.result);

        let camel_case = method.name.to_case(Case::Camel);

        workspace_members.push(format!(
            "\t{camel_case}(params: {params}): Promise<{result}>;"
        ));

        impl_members.push(format!(
            "\t\t{camel_case}(params) {{\n\t\t\treturn transport.request(\"pgls/{}\", params);\n\t\t}}",
            method.name
        ));
    }

    let mut output = String::new();
    output.push_str("// Generated file, do not edit by hand, see `xtask/codegen`\n");
    output.push_str("import type { Transport } from \"./transport\";\n");

    for (decl, description) in &declarations {
        if let Some(description) = description {
            output.push_str(&format!("/**\n * {description}\n */\n"));
        }
        output.push_str(&format!("export {decl}\n"));
    }

    output.push_str("export type Configuration = PartialConfiguration;\n");

    workspace_members.push("\tdestroy(): void;".to_string());
    output.push_str("export interface Workspace {\n");
    for member in &workspace_members {
        output.push_str(member);
        output.push('\n');
    }
    output.push_str("}\n");

    impl_members.push("\t\tdestroy() {\n\t\t\ttransport.destroy();\n\t\t}".to_string());
    output.push_str("export function createWorkspace(transport: Transport): Workspace {\n");
    output.push_str("\treturn {\n");
    output.push_str(&impl_members.join(",\n"));
    output.push_str(",\n");
    output.push_str("\t};\n");
    output.push_str("}\n");

    update(&bindings_path_postgrestools, &output, mode)?;
    update(&bindings_path_postgres_language_server, &output, mode)?;

    Ok(())
}
