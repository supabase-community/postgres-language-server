//! The names of migration files. Supabase, dbmate and goose name them the same way,
//! `<number>_<name>.sql`.

use std::path::Path;

use crate::{FileInfo, Role, Version};

/// A file in the migrations folder is a migration. Its version is the number that starts its
/// name, `<version>_<name>.sql`, or the name of its folder, `<version>_<name>/up.sql`.
pub(crate) fn numbered(
    path: &Path,
    migrations_dir: &Path,
    version: impl Fn(&str) -> Option<Version>,
) -> FileInfo {
    if !is_in_migrations_dir(path, migrations_dir) {
        return FileInfo {
            role: Role::Other,
            version: None,
        };
    }

    // In the "root" pattern, all files are directly within the migrations directory, and their
    // names follow <timestamp>_<name>.sql. This is used by Supabase. In the "subdirectory"
    // pattern, each migration is in a subdirectory named <timestamp>_<name>. This is used by
    // Prisma and Drizzle.
    let version_of = |name: Option<&std::ffi::OsStr>| {
        let (number, _name) = name?.to_str()?.split_once('_')?;
        version(number)
    };
    FileInfo {
        role: Role::Migration,
        version: version_of(path.file_name())
            .or_else(|| version_of(path.parent().and_then(Path::file_name))),
    }
}

/// A number with leading zeros, like `000201` or `20240101120000`.
pub(crate) fn number(text: &str) -> Option<Version> {
    text.trim_start_matches('0')
        .parse()
        .ok()
        .map(Version::Number)
}

/// Whether the path is inside the migrations directory.
fn is_in_migrations_dir(path: &Path, migrations_dir: &Path) -> bool {
    if path.starts_with(migrations_dir) {
        return true;
    }
    path.canonicalize()
        .ok()
        .and_then(|canonical_child| {
            migrations_dir
                .canonicalize()
                .ok()
                .map(|canonical_dir| canonical_child.starts_with(&canonical_dir))
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    fn version_of(path: &Path, migrations_dir: &Path) -> Option<Version> {
        let file = numbered(path, migrations_dir, number);
        assert_eq!(file.role == Role::Migration, file.version.is_some());
        file.version
    }

    #[test]
    fn test_get_migration_root_pattern() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let path = migrations_dir.join("1234567890_create_users.sql");
        fs::write(&path, "").unwrap();

        assert_eq!(
            version_of(&path, &migrations_dir),
            Some(Version::Number(1234567890))
        );
    }

    #[test]
    fn test_get_migration_subdirectory_pattern() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let subdir = migrations_dir.join("1234567890_create_users");
        fs::create_dir(&subdir).unwrap();
        let path = subdir.join("up.sql");
        fs::write(&path, "").unwrap();

        assert_eq!(
            version_of(&path, &migrations_dir),
            Some(Version::Number(1234567890))
        );
    }

    #[test]
    fn test_get_migration_prefix_number() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let path = migrations_dir.join("000201_a_migration.sql");
        fs::write(&path, "").unwrap();

        assert_eq!(
            version_of(&path, &migrations_dir),
            Some(Version::Number(201))
        );
    }

    #[test]
    fn test_get_migration_not_timestamp_in_filename() {
        let migrations_dir = PathBuf::from("/tmp/migrations");
        let path = migrations_dir.join("not_a_migration.sql");

        let file = numbered(&path, &migrations_dir, number);
        assert_eq!(file.role, Role::Migration);
        assert_eq!(file.version, None);
    }

    #[test]
    fn test_get_migration_outside_migrations_dir() {
        let migrations_dir = PathBuf::from("/tmp/migrations");
        let path = PathBuf::from("/tmp/other/1234567890_create_users.sql");

        let file = numbered(&path, &migrations_dir, number);
        assert_eq!(file.role, Role::Other);
        assert_eq!(file.version, None);
    }
}
