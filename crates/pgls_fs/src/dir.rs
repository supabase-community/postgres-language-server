use directories::{BaseDirs, ProjectDirs};
use std::{env, fs, path::PathBuf};
use tracing::warn;

/// The directory inside the user's configuration directory that holds the user configuration.
const USER_CONFIG_DIR_NAME: &str = "postgres-language-server";

pub fn ensure_cache_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "supabase-community", "pgls") {
        // Linux: /home/alice/.cache/pgls
        // Win: C:\Users\Alice\AppData\Local\supabase-community\pgls\cache
        // Mac: /Users/Alice/Library/Caches/dev.supabase-community.pgls
        let cache_dir = proj_dirs.cache_dir().to_path_buf();
        if let Err(err) = fs::create_dir_all(&cache_dir) {
            let temp_dir = env::temp_dir();
            warn!(
                "Failed to create local cache directory {cache_dir:?} due to error: {err}, fallback to {temp_dir:?}"
            );
            temp_dir
        } else {
            cache_dir
        }
    } else {
        env::temp_dir()
    }
}

/// Returns the home directory of the current user.
pub fn home_dir() -> Option<PathBuf> {
    BaseDirs::new().map(|base_dirs| base_dirs.home_dir().to_path_buf())
}

/// Returns the directories the user configuration is looked up in, in order of precedence.
///
/// - Linux: `$XDG_CONFIG_HOME/postgres-language-server`, or `~/.config/postgres-language-server`
/// - macOS: the same as Linux, then `~/Library/Application Support/postgres-language-server`
/// - Windows: `%APPDATA%\postgres-language-server`
pub fn user_config_dirs() -> Vec<PathBuf> {
    let Some(base_dirs) = BaseDirs::new() else {
        return Vec::new();
    };

    let native = base_dirs.config_dir().to_path_buf();
    // Command line tools on macOS conventionally read `~/.config`, which is where users look
    // first. `directories` only knows the native `~/Library/Application Support`.
    #[cfg(target_os = "macos")]
    let dirs = vec![
        xdg_config_home(env::var_os("XDG_CONFIG_HOME"), base_dirs.home_dir()),
        native,
    ];
    #[cfg(not(target_os = "macos"))]
    let dirs = vec![native];

    dirs.into_iter()
        .map(|dir| dir.join(USER_CONFIG_DIR_NAME))
        .collect()
}

/// `$XDG_CONFIG_HOME` if it is an absolute path, as the XDG specification requires, or else
/// `~/.config`.
#[cfg(any(target_os = "macos", test))]
fn xdg_config_home(xdg_config_home: Option<std::ffi::OsString>, home: &std::path::Path) -> PathBuf {
    xdg_config_home
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .unwrap_or_else(|| home.join(".config"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn xdg_config_home_defaults_to_dot_config() {
        let home = Path::new("/home/alice");
        assert_eq!(xdg_config_home(None, home), home.join(".config"));
    }

    #[test]
    fn xdg_config_home_uses_an_absolute_override() {
        let home = Path::new("/home/alice");
        let config_home = env::temp_dir().join("xdg");
        assert_eq!(
            xdg_config_home(Some(config_home.clone().into()), home),
            config_home
        );
    }

    #[test]
    fn xdg_config_home_ignores_a_relative_override() {
        let home = Path::new("/home/alice");
        assert_eq!(
            xdg_config_home(Some("config".into()), home),
            home.join(".config")
        );
    }
}
