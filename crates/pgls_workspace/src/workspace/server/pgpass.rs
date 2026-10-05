//! Looks up passwords in the password file, as libpq does.
//!
//! sqlx reads the password file too, but only while it builds the connection options from the
//! environment or a URL, and it does not tell whether it found a password.

use std::path::{Path, PathBuf};

/// The connection a password is looked up for.
pub(crate) struct PassfileTarget<'a> {
    pub host: &'a str,
    pub port: u16,
    pub database: &'a str,
    pub username: &'a str,
}

/// Looks up the password for the target in the password file: `PGPASSFILE`, or else
/// `~/.pgpass` (`%APPDATA%\postgresql\pgpass.conf` on Windows).
///
/// Port of the lookup in `pqConnectOptions2`:
/// https://github.com/postgres/postgres/blob/REL_18_6/src/interfaces/libpq/fe-connect.c#L1423
pub(crate) fn password_from_passfile(target: &PassfileTarget) -> Option<String> {
    password_from_file(&passfile_path()?, target)
}

fn passfile_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PGPASSFILE").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(path));
    }

    #[cfg(windows)]
    let path = std::env::var_os("APPDATA")
        .map(|dir| PathBuf::from(dir).join("postgresql").join("pgpass.conf"));
    #[cfg(not(windows))]
    let path = std::env::home_dir().map(|dir| dir.join(".pgpass"));

    path
}

/// Returns the password of the first entry in the password file at `path` that matches the
/// target, or `None` when there is none or the file cannot be used.
///
/// Port of `passwordFromFile`:
/// https://github.com/postgres/postgres/blob/REL_18_6/src/interfaces/libpq/fe-connect.c#L7906
pub(crate) fn password_from_file(path: &Path, target: &PassfileTarget) -> Option<String> {
    if target.database.is_empty() || target.username.is_empty() {
        return None;
    }

    // 'localhost' matches an empty host.
    let host = if target.host.is_empty() {
        "localhost"
    } else {
        target.host
    };
    let port = target.port.to_string();

    let content = read_passfile(path)?;
    content
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.starts_with(b"#"))
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
        .filter(|line| !line.is_empty())
        .find_map(|line| {
            let rest = match_field(line, host.as_bytes())?;
            let rest = match_field(rest, port.as_bytes())?;
            let rest = match_field(rest, target.database.as_bytes())?;
            let rest = match_field(rest, target.username.as_bytes())?;
            Some(unescape_password(rest))
        })
        .and_then(|password| String::from_utf8(password).ok())
}

fn read_passfile(path: &Path) -> Option<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = std::fs::metadata(path).ok()?;
        if !metadata.is_file() {
            tracing::warn!("Password file {} is not a plain file", path.display());
            return None;
        }
        if metadata.permissions().mode() & 0o077 != 0 {
            tracing::warn!(
                "Password file {} has group or world access; permissions should be u=rw (0600) or less",
                path.display()
            );
            return None;
        }
    }

    std::fs::read(path).ok()
}

/// Matches the next field of an entry, which is `*` or the token with `:` and `\` escaped by
/// a backslash, and returns the rest of the entry.
///
/// Port of `pwdfMatchesString`:
/// https://github.com/postgres/postgres/blob/REL_18_6/src/interfaces/libpq/fe-connect.c#L7869
fn match_field<'a>(entry: &'a [u8], mut token: &[u8]) -> Option<&'a [u8]> {
    if let Some(rest) = entry.strip_prefix(b"*:") {
        return Some(rest);
    }

    let mut bytes = entry.iter().enumerate();
    while let Some((index, &byte)) = bytes.next() {
        let (index, byte, escaped) = if byte == b'\\' {
            let (index, &byte) = bytes.next()?;
            (index, byte, true)
        } else {
            (index, byte, false)
        };

        if byte == b':' && !escaped && token.is_empty() {
            return Some(&entry[index + 1..]);
        }
        let (&expected, rest) = token.split_first()?;
        if byte != expected {
            return None;
        }
        token = rest;
    }

    None
}

/// The password is the rest of the entry up to an unescaped `:`.
fn unescape_password(field: &[u8]) -> Vec<u8> {
    let mut password = Vec::with_capacity(field.len());
    let mut index = 0;
    while index < field.len() && field[index] != b':' {
        if field[index] == b'\\' && index + 1 < field.len() {
            index += 1;
        }
        password.push(field[index]);
        index += 1;
    }
    password
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET: PassfileTarget<'static> = PassfileTarget {
        host: "db.example.com",
        port: 5432,
        database: "app",
        username: "alice",
    };

    fn passfile(content: &str) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(file.path(), std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        file
    }

    fn lookup(content: &str, target: &PassfileTarget) -> Option<String> {
        let file = passfile(content);
        password_from_file(file.path(), target)
    }

    #[test]
    fn returns_the_password_of_the_first_matching_entry() {
        let content = "\
db.example.com:5432:app:bob:not-alice
db.example.com:5433:app:alice:other-port
other.example.com:5432:app:alice:other-host
db.example.com:5432:other:alice:other-database
db.example.com:5432:app:alice:secret
db.example.com:5432:app:alice:second-match
";
        assert_eq!(lookup(content, &TARGET).as_deref(), Some("secret"));
    }

    #[test]
    fn wildcards_match_any_value() {
        assert_eq!(
            lookup("*:*:*:alice:secret\n", &TARGET).as_deref(),
            Some("secret")
        );
        assert_eq!(
            lookup("*:*:*:*:secret\n", &TARGET).as_deref(),
            Some("secret")
        );
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let content = "# db.example.com:5432:app:alice:commented\n\n*:*:*:*:secret\r\n";
        assert_eq!(lookup(content, &TARGET).as_deref(), Some("secret"));
        assert_eq!(
            lookup("*:*:*:*:trailing space \n", &TARGET).as_deref(),
            Some("trailing space ")
        );
    }

    #[test]
    fn unescapes_colons_and_backslashes() {
        let target = PassfileTarget {
            database: "a:b",
            username: "c\\d",
            ..TARGET
        };
        assert_eq!(
            lookup("*:*:a\\:b:c\\\\d:pass\\:wo\\\\rd\n", &target).as_deref(),
            Some("pass:wo\\rd")
        );
    }

    #[test]
    fn password_ends_at_an_unescaped_colon() {
        assert_eq!(
            lookup("*:*:*:*:secret:trailing\n", &TARGET).as_deref(),
            Some("secret")
        );
    }

    #[test]
    fn empty_host_matches_localhost() {
        let target = PassfileTarget { host: "", ..TARGET };
        assert_eq!(
            lookup("localhost:5432:app:alice:secret\n", &target).as_deref(),
            Some("secret")
        );
    }

    #[test]
    fn host_is_compared_literally() {
        let target = PassfileTarget {
            host: "127.0.0.1",
            ..TARGET
        };
        assert_eq!(lookup("localhost:5432:app:alice:secret\n", &target), None);
    }

    #[test]
    fn missing_file_has_no_password() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            password_from_file(&dir.path().join("pgpass"), &TARGET),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn ignores_a_file_others_can_access() {
        use std::os::unix::fs::PermissionsExt;

        let file = passfile("*:*:*:*:secret\n");
        std::fs::set_permissions(file.path(), std::fs::Permissions::from_mode(0o640)).unwrap();
        assert_eq!(password_from_file(file.path(), &TARGET), None);
    }

    #[cfg(unix)]
    #[test]
    fn ignores_a_path_that_is_not_a_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(password_from_file(dir.path(), &TARGET), None);
    }
}
