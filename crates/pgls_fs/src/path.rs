use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

/// A path of a file of the workspace.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgLSPath {
    path: PathBuf,
    /// Whether this path (usually a file) was fixed as a result of a format/lint/check command with the `--write` filag.
    was_written: bool,
}

impl Deref for PgLSPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl PgLSPath {
    pub fn new(path_to_file: impl Into<PathBuf>) -> Self {
        Self {
            path: path_to_file.into(),
            was_written: false,
        }
    }

    /// Creates a new [PgLSPath], marked as fixed
    pub fn to_written(&self) -> Self {
        Self {
            path: self.path.clone(),
            was_written: true,
        }
    }

    pub fn was_written(&self) -> bool {
        self.was_written
    }
}

/// Normalizes a path, resolving `..` and `.` segments without requiring the path to exist or
/// following symlinks.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    let mut prefix_component = None;
    let mut is_absolute = false;

    for component in path.components() {
        match component {
            std::path::Component::Prefix(_prefix) => {
                prefix_component = Some(component);
                components.clear();
            }
            std::path::Component::RootDir => {
                is_absolute = true;
                components.clear();
            }
            std::path::Component::ParentDir => {
                if !components.is_empty() {
                    components.pop();
                } else if !is_absolute && prefix_component.is_none() {
                    // Only keep parent dir if we're not absolute and have no prefix
                    components.push(component.as_os_str());
                }
            }
            std::path::Component::Normal(c) => {
                components.push(c);
            }
            std::path::Component::CurDir => {
                // Skip current directory components
            }
        }
    }

    let mut result = PathBuf::new();

    // Add prefix component (like C: on Windows)
    if let Some(prefix) = prefix_component {
        result.push(prefix.as_os_str());
    }

    // Add root directory if path is absolute
    if is_absolute {
        result.push(std::path::Component::RootDir.as_os_str());
    }

    // Add normalized components
    for component in components {
        result.push(component);
    }

    // Handle edge cases
    if result.as_os_str().is_empty() {
        if prefix_component.is_some() || is_absolute {
            // This shouldn't happen with proper input, but fallback to original path's root
            return path
                .ancestors()
                .last()
                .unwrap_or(Path::new(""))
                .to_path_buf();
        } else {
            return PathBuf::from(".");
        }
    }

    result
}
