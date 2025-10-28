use enumflags2::{BitFlags, bitflags};
use smallvec::SmallVec;
use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs::File,
    fs::read_to_string,
    io,
    io::Write,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use crate::ConfigName;

/// The priority of the file
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(u8)]
#[bitflags]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
// NOTE: The order of the variants is important, the one on the top has the highest priority
pub enum FileKind {
    /// A configuration file has the highest priority. It's usually `postgres-language-server.jsonc`
    ///
    /// Other third-party configuration files might be added in the future
    Config,
    /// An ignore file, like `.gitignore`
    Ignore,
    /// Files that are required to be inspected before handling other files.
    Inspectable,
    /// A file to handle has the lowest priority. It's usually a traversed file, or a file opened by the LSP
    #[default]
    Handleable,
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(
        from = "smallvec::SmallVec<[FileKind; 5]>",
        into = "smallvec::SmallVec<[FileKind; 5]>"
    )
)]
pub struct FileKinds(BitFlags<FileKind>);

impl From<SmallVec<[FileKind; 5]>> for FileKinds {
    fn from(value: SmallVec<[FileKind; 5]>) -> Self {
        value
            .into_iter()
            .fold(FileKinds::default(), |mut acc, kind| {
                acc.insert(kind);
                acc
            })
    }
}

impl From<FileKinds> for SmallVec<[FileKind; 5]> {
    fn from(value: FileKinds) -> Self {
        value.iter().collect()
    }
}

impl Deref for FileKinds {
    type Target = BitFlags<FileKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FileKinds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<FileKind> for FileKinds {
    fn from(flag: FileKind) -> Self {
        Self(BitFlags::from(flag))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgLSPath {
    path: PathBuf,
    /// Determines the kind of the file inside Postgres Language Server. Some files are considered as configuration files, others as manifest files, and others as files to handle
    kind: FileKinds,
    /// Whether this path (usually a file) was fixed as a result of a format/lint/check command with the `--write` filag.
    was_written: bool,
}

impl Deref for PgLSPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl PartialOrd for PgLSPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PgLSPath {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.kind.cmp(&other.kind) {
            Ordering::Equal => self.path.cmp(&other.path),
            ordering => ordering,
        }
    }
}

impl PgLSPath {
    pub fn new(path_to_file: impl Into<PathBuf>) -> Self {
        let path = path_to_file.into();
        let kind = path.file_name().map(Self::priority).unwrap_or_default();
        Self {
            path,
            kind,
            was_written: false,
        }
    }

    pub fn new_written(path_to_file: impl Into<PathBuf>) -> Self {
        let path = path_to_file.into();
        let kind = path.file_name().map(Self::priority).unwrap_or_default();
        Self {
            path,
            kind,
            was_written: true,
        }
    }

    /// Creates a new [PgLSPath], marked as fixed
    pub fn to_written(&self) -> Self {
        Self {
            path: self.path.clone(),
            kind: self.kind.clone(),
            was_written: true,
        }
    }

    pub fn was_written(&self) -> bool {
        self.was_written
    }

    /// Accepts a file opened in read mode and saves into it
    pub fn save(&mut self, content: &str) -> Result<(), std::io::Error> {
        let mut file_to_write = File::create(&self.path).unwrap();
        // TODO: handle error with diagnostic
        file_to_write.write_all(content.as_bytes())
    }

    /// Returns the contents of a file, if it exists
    ///
    /// ## Error
    /// If Postgres Tools doesn't have permissions to read the file
    pub fn get_buffer_from_file(&mut self) -> String {
        // we assume we have permissions
        read_to_string(&self.path).expect("cannot read the file to format")
    }

    /// Small wrapper for [read_to_string]
    pub fn read_to_string(&self) -> io::Result<String> {
        let path = self.path.as_path();
        read_to_string(path)
    }

    /// The priority of the file.
    /// - `postgres-language-server.jsonc` has the highest priority
    /// - Other files are considered as files to handle
    fn priority(file_name: &OsStr) -> FileKinds {
        if ConfigName::file_names().iter().any(|f| *f == file_name) {
            FileKind::Config.into()
        } else {
            FileKind::Handleable.into()
        }
    }

    pub fn is_config(&self) -> bool {
        self.kind.contains(FileKind::Config)
    }

    pub fn is_ignore(&self) -> bool {
        self.kind.contains(FileKind::Ignore)
    }

    pub fn is_to_inspect(&self) -> bool {
        self.kind.contains(FileKind::Inspectable)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for FileKinds {
    fn schema_name() -> String {
        String::from("FileKind")
    }

    fn json_schema(r#gen: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        <Vec<FileKind>>::json_schema(r#gen)
    }
}
