use directories::ProjectDirs;
use std::{env, fs, path::PathBuf};
use tracing::warn;

pub fn ensure_cache_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "supabase-community", "pgt") {
        // Linux: /home/alice/.cache/pgt
        // Win: C:\Users\Alice\AppData\Local\supabase-community\pgt\cache
        // Mac: /Users/Alice/Library/Caches/dev.supabase-community.pgt
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
