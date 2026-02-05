use xtask::Mode::Overwrite;
use xtask::{project_root, pushd, Result};

use xtask_codegen::{
    generate_analyser, generate_bindings, generate_crate, generate_new_analyser_rule,
    generate_pglinter, generate_rules_configuration, generate_splinter, task_command, TaskCommand,
};

fn main() -> Result<()> {
    let _d = pushd(project_root());
    let result = task_command().fallback_to_usage().run();

    match result {
        TaskCommand::Analyser => {
            generate_analyser()?;
        }
        TaskCommand::NewCrate { name } => {
            generate_crate(name)?;
        }
        TaskCommand::NewRule {
            name,
            category,
            group,
            severity,
        } => {
            generate_new_analyser_rule(category, &name, &group, severity);
        }
        TaskCommand::Configuration => {
            generate_rules_configuration(Overwrite)?;
        }
        TaskCommand::Bindings => {
            generate_bindings(Overwrite)?;
        }
        TaskCommand::Splinter => {
            generate_splinter()?;
        }
        TaskCommand::Pglinter => {
            generate_pglinter()?;
        }
    }

    Ok(())
}
