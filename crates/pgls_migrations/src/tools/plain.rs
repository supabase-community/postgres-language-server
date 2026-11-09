//! Migration files without a tool: every file is one migration.

use std::path::Path;

use pgls_fs::FileSystem;

use crate::{Detected, FileInfo, MigrationTool, Root, Script, ToolId, Version, naming};

pub struct Plain;

impl MigrationTool for Plain {
    fn id(&self) -> ToolId {
        ToolId::Plain
    }

    fn name(&self) -> &'static str {
        "plain SQL"
    }

    /// Plain SQL files have no sign of a project.
    fn detect(&self, _fs: &dyn FileSystem, _root: &Root) -> Option<Detected> {
        None
    }

    fn is_tool_directory(&self, _fs: &dyn FileSystem, _directory: &Path) -> bool {
        false
    }

    fn default_dir(&self) -> Option<&'static str> {
        None
    }

    fn file(&self, path: &Path, migrations_dir: &Path) -> FileInfo {
        naming::numbered(path, migrations_dir, |text| self.version(text))
    }

    fn version(&self, text: &str) -> Option<Version> {
        naming::number(text)
    }

    /// The file is one migration, whatever its comments say.
    fn read(&self, _text: &str) -> Script {
        Script::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ToolId,
        tests::{marker_problems, sections},
    };

    #[test]
    fn no_markers() {
        let text = "-- migrate:up\nselect 1;\n-- +goose Down\n";
        assert!(sections(text, ToolId::Plain).is_empty());
        assert!(marker_problems(text, ToolId::Plain).is_empty());
        assert!(sections("select 1;", ToolId::Dbmate).is_empty());
    }
}
