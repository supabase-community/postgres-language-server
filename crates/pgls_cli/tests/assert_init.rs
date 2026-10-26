use assert_cmd::cargo_bin_cmd;
use insta::assert_snapshot;
use serde_json::Value;
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "postgres-language-server.jsonc";

#[test]
fn init_without_supabase_project_uses_default_connection() {
    let dir = tempfile::tempdir().unwrap();

    let stdout = run_init(dir.path());

    assert!(!stdout.contains("Supabase"), "{stdout}");
    assert_eq!(read_db_config(dir.path()), db_config(5432));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn init_with_supabase_project_uses_configured_port() {
    let dir = tempfile::tempdir().unwrap();
    write_supabase_config(
        dir.path(),
        "project_id = \"test\"\n\n[db]\nport = 6543\nshadow_port = 54320\n",
    );

    let stdout = run_init(dir.path());

    assert_snapshot!(stdout);
    assert_eq!(read_db_config(dir.path()), db_config(6543));
}

#[test]
fn init_with_supabase_project_without_db_section_uses_default_port() {
    let dir = tempfile::tempdir().unwrap();
    write_supabase_config(dir.path(), "project_id = \"test\"\n");

    let stdout = run_init(dir.path());

    assert!(stdout.contains("Supabase project detected"), "{stdout}");
    assert!(!stdout.contains("Could not read"), "{stdout}");
    assert_eq!(read_db_config(dir.path()), db_config(54322));
}

#[test]
fn init_with_invalid_supabase_config_uses_default_port() {
    let dir = tempfile::tempdir().unwrap();
    write_supabase_config(dir.path(), "[db\nport = 6543\n");

    let stdout = run_init(dir.path());

    assert!(
        stdout.contains("Could not read the database port"),
        "{stdout}"
    );
    assert!(stdout.contains("Supabase project detected"), "{stdout}");
    assert_eq!(read_db_config(dir.path()), db_config(54322));
}

#[test]
fn init_with_supabase_project_sets_the_migrations_dir() {
    let dir = tempfile::tempdir().unwrap();
    write_supabase_config(dir.path(), "project_id = \"test\"\n");

    run_init(dir.path());

    let config = read_config(dir.path());
    assert_eq!(config["migrations"]["migrationsDir"], "supabase/migrations");
    // Executing statements stays opt-in.
    assert!(
        config["db"]["allowStatementExecutionsAgainst"]
            .as_array()
            .is_none_or(|patterns| patterns.is_empty()),
        "{config}"
    );
}

#[test]
fn init_in_a_monorepo_detects_the_supabase_project_below_it() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("apps/web")).unwrap();
    fs::create_dir_all(dir.path().join("node_modules/pkg/supabase")).unwrap();
    fs::write(
        dir.path().join("node_modules/pkg/supabase/config.toml"),
        "[db]\nport = 1\n",
    )
    .unwrap();
    write_supabase_config(&dir.path().join("apps/web"), "[db]\nport = 6543\n");

    let stdout = run_init(dir.path());

    assert!(
        stdout.contains("Supabase project detected in apps/web/supabase"),
        "{stdout}"
    );
    assert_eq!(read_db_config(dir.path()), db_config(6543));
    assert_eq!(
        read_config(dir.path())["migrations"]["migrationsDir"],
        "apps/web/supabase/migrations"
    );
}

#[test]
fn init_with_several_supabase_projects_leaves_the_connection_alone() {
    let dir = tempfile::tempdir().unwrap();
    for project in ["one", "two"] {
        fs::create_dir_all(dir.path().join(project)).unwrap();
        write_supabase_config(&dir.path().join(project), "[db]\nport = 6543\n");
    }

    let stdout = run_init(dir.path());

    assert!(
        stdout.contains("Found several Supabase projects (one/supabase, two/supabase)"),
        "{stdout}"
    );
    assert_eq!(read_db_config(dir.path()), db_config(5432));
    assert!(read_config(dir.path())["migrations"].is_null());
}

fn run_init(dir: &Path) -> String {
    let output = cargo_bin_cmd!("postgres-language-server")
        .current_dir(dir)
        .arg("init")
        .output()
        .expect("failed to run CLI");
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn write_supabase_config(dir: &Path, contents: &str) {
    fs::create_dir(dir.join("supabase")).unwrap();
    fs::write(dir.join("supabase/config.toml"), contents).unwrap();
}

fn read_config(dir: &Path) -> Value {
    let contents = fs::read_to_string(dir.join(CONFIG_FILE)).unwrap();
    serde_json::from_str(&contents).unwrap()
}

fn read_db_config(dir: &Path) -> Value {
    let config = read_config(dir);
    let db = &config["db"];
    serde_json::json!({
        "host": db["host"],
        "port": db["port"],
        "username": db["username"],
        "password": db["password"],
        "database": db["database"],
    })
}

fn db_config(port: u16) -> Value {
    serde_json::json!({
        "host": "127.0.0.1",
        "port": port,
        "username": "postgres",
        "password": "postgres",
        "database": "postgres",
    })
}
