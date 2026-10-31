use assert_cmd::cargo_bin_cmd;
use insta::assert_snapshot;
#[cfg(target_os = "linux")]
use sqlx::PgPool;
use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::process::ExitStatus;
const CONFIG_PATH: &str = "tests/fixtures/postgres-language-server.jsonc";

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_default_reporter_snapshot() {
    assert_snapshot!(run_check(&["tests/fixtures/test.sql"]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_github_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "github",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_gitlab_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "gitlab",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_junit_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "junit",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_json_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "json",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_json_pretty_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "json-pretty",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_summary_reporter_snapshot() {
    assert_snapshot!(run_check(&[
        "--reporter",
        "summary",
        "tests/fixtures/test.sql"
    ]));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_stdin_snapshot() {
    assert_snapshot!(run_check_with(
        &[
            "--config-path",
            CONFIG_PATH,
            "--stdin-file-path",
            "virtual.sql",
            "--log-level",
            "none"
        ],
        Some("alter tqjable stdin drop column id;\n"),
        None
    ));
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_indented_cte_after_blank_line_does_not_panic() {
    let output = run_check_with(
        &[
            "--config-path",
            CONFIG_PATH,
            "--stdin-file-path",
            "repro.sql",
            "--log-level",
            "none",
        ],
        Some("WITH\n\n x AS (SELECT 1 AS a)\nSELECT x.a FROM x;\n"),
        None,
    );

    assert!(!output.contains("Encountered an unexpected error"));
    assert!(!output.contains("assertion failed"));
    assert_snapshot!(output);
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "snapshot expectations only validated on unix-like platforms"
)]
fn check_directory_traversal_snapshot() {
    let project_dir = Path::new("tests/fixtures/traversal");
    assert_snapshot!(run_check_with(
        &["--diagnostic-level", "info", "."],
        None,
        Some(project_dir)
    ));
}

#[cfg(target_os = "linux")]
fn get_database_url(pool: &PgPool) -> String {
    let opts = pool.connect_options();
    format!(
        "postgres://postgres:postgres@{}:{}/{}",
        opts.get_host(),
        opts.get_port(),
        opts.get_database().unwrap_or("postgres")
    )
}

#[cfg(target_os = "linux")]
fn create_temp_sql_file(contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "pgls-tls-{}-{}.sql",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));

    std::fs::write(&path, contents).expect("failed to write temporary SQL file");
    path
}

#[cfg(target_os = "linux")]
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn check_accepts_tls_connection_strings(test_db: PgPool) {
    let sql_file = create_temp_sql_file("select 1;\n");
    let connection_string = format!(
        "{}?sslmode=require&channel_binding=require",
        get_database_url(&test_db)
    );

    let mut cmd = cargo_bin_cmd!("postgres-language-server");
    let output = cmd
        .args([
            "check",
            sql_file
                .to_str()
                .expect("temporary SQL file path should be valid UTF-8"),
            "--connection-string",
            &connection_string,
            "--log-level",
            "none",
        ])
        .output()
        .expect("failed to run CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "expected TLS-backed check to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("TLS upgrade required") && !stderr.contains("TLS upgrade required"),
        "TLS support error should not appear\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Checked 1 file"),
        "expected successful CLI output\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    std::fs::remove_file(sql_file).expect("failed to remove temporary SQL file");
}

#[cfg(target_os = "linux")]
const UNKNOWN_COLUMN_PATH: &str = "tests/fixtures/unknown_column.sql";

#[cfg(target_os = "linux")]
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn check_fails_when_database_login_fails(test_db: PgPool) {
    let opts = test_db.connect_options();
    let port = opts.get_port().to_string();

    assert_snapshot!(run_check_without_db_env(
        &[
            "--host",
            opts.get_host(),
            "--port",
            &port,
            "--username",
            "postgres",
            "--password",
            "wrong-password",
            "--database",
            opts.get_database().unwrap_or("postgres"),
            UNKNOWN_COLUMN_PATH,
        ],
        &[],
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn check_fails_when_database_is_unreachable() {
    // Nothing listens on port 1.
    let output = run_check_without_db_env(
        &[
            "--host",
            "127.0.0.1",
            "--port",
            "1",
            "--conn_timeout_secs",
            "1",
            UNKNOWN_COLUMN_PATH,
        ],
        &[],
    );

    assert!(output.starts_with("status: failure"), "{output}");
    assert!(output.contains("database/connection"), "{output}");
}

#[cfg(target_os = "linux")]
#[test]
fn check_fails_on_invalid_connection_string() {
    let output = run_check_without_db_env(
        &[
            "--connection-string",
            "postgres://postgres:hunter2@localhost:not-a-port/postgres",
            UNKNOWN_COLUMN_PATH,
        ],
        &[],
    );

    assert!(!output.contains("hunter2"), "{output}");
    assert_snapshot!(output);
}

#[cfg(target_os = "linux")]
#[test]
fn check_fails_on_invalid_database_url() {
    let output = run_check_without_db_env(
        &[UNKNOWN_COLUMN_PATH],
        &[("DATABASE_URL", "host=localhost password=hunter2")],
    );

    assert!(output.starts_with("status: failure"), "{output}");
    assert!(output.contains("database/connection"), "{output}");
    assert!(output.contains("Invalid `db.connectionString`"), "{output}");
    assert!(!output.contains("hunter2"), "{output}");
}

#[cfg(target_os = "linux")]
#[test]
fn check_without_database_succeeds_with_disable_db() {
    let output = run_check_without_db_env(
        &[
            "--host",
            "127.0.0.1",
            "--port",
            "1",
            "--conn_timeout_secs",
            "1",
            "--disable-db",
            UNKNOWN_COLUMN_PATH,
        ],
        &[],
    );

    assert!(output.starts_with("status: success"), "{output}");
    assert!(!output.contains("database/connection"), "{output}");
}

#[cfg(target_os = "linux")]
#[test]
fn check_without_database_succeeds_with_disable_connection_in_config() {
    let mut cmd = cargo_bin_cmd!("postgres-language-server");
    let output = cmd
        .env_remove("DATABASE_URL")
        .env_remove("PGHOST")
        .args([
            "check",
            "--config-path",
            "tests/fixtures/disable_connection/postgres-language-server.jsonc",
            "--log-level",
            "none",
            UNKNOWN_COLUMN_PATH,
        ])
        .output()
        .expect("failed to run CLI");
    let output = normalize_output(
        output.status,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    );

    assert!(output.starts_with("status: success"), "{output}");
    assert!(!output.contains("database/connection"), "{output}");
}

#[cfg(target_os = "linux")]
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn check_reads_the_password_from_pgpassfile(test_db: PgPool) {
    use std::os::unix::fs::PermissionsExt;

    let opts = test_db.connect_options();
    let host = opts.get_host().to_string();
    let port = opts.get_port().to_string();
    let database = opts.get_database().unwrap_or("postgres").to_string();
    let role = format!("pgls_pgpass_{}", std::process::id());
    sqlx::query(&format!("drop role if exists {role}"))
        .execute(&test_db)
        .await
        .unwrap();
    sqlx::query(&format!(
        "create role {role} login password 'pgpass:secret'"
    ))
    .execute(&test_db)
    .await
    .unwrap();

    let passfile = std::env::temp_dir().join(format!("{role}.pgpass"));
    std::fs::write(
        &passfile,
        format!("# comment\n{host}:{port}:{database}:{role}:pgpass\\:secret\n"),
    )
    .unwrap();
    std::fs::set_permissions(&passfile, std::fs::Permissions::from_mode(0o600)).unwrap();

    let output = run_check_without_db_env(
        &[
            "--host",
            &host,
            "--port",
            &port,
            "--username",
            &role,
            "--database",
            &database,
            UNKNOWN_COLUMN_PATH,
        ],
        &[("PGPASSFILE", passfile.to_str().unwrap())],
    );

    std::fs::remove_file(&passfile).unwrap();
    sqlx::query(&format!("drop role {role}"))
        .execute(&test_db)
        .await
        .unwrap();

    // The column check needs the database schema, so it only runs after a successful login.
    assert!(!output.contains("database/connection"), "{output}");
    assert!(output.contains("lint/unknownColumn"), "{output}");
}

/// Each file uses the nearest configuration file at or above it, below the working directory.
#[cfg(target_os = "linux")]
#[test]
fn check_uses_the_nearest_configuration_file() {
    let root = tempfile::tempdir().unwrap();
    let write = |path: &str, content: &str| {
        let path = root.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    };
    write(
        "postgres-language-server.jsonc",
        r#"{ "db": { "disableConnection": true } }"#,
    );
    write(
        "db1/postgres-language-server.jsonc",
        r#"{ "db": { "disableConnection": true }, "linter": { "enabled": false } }"#,
    );
    write(
        "db2/postgres-language-server.jsonc",
        r#"{
            "extends": ["../postgres-language-server.jsonc"],
            "linter": { "rules": { "banDropColumn": "error" } }
        }"#,
    );
    for file in ["a.sql", "db1/a.sql", "db2/schema/a.sql"] {
        write(file, "alter table users drop column name;\n");
    }

    let check = |paths: &[&str]| {
        let mut cmd = cargo_bin_cmd!("postgres-language-server");
        for var in ["DATABASE_URL", "PGHOST", "PGPORT", "PGUSER", "PGDATABASE"] {
            cmd.env_remove(var);
        }
        let output = cmd
            .current_dir(root.path())
            .args(["check", "--reporter", "github", "--log-level", "none"])
            .args(paths)
            .output()
            .expect("failed to run CLI");
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    let annotation =
        |level: &str, file: &str| format!("::{level} title=lint/banDropColumn,file={file},");

    // The traversal of the working directory passes absolute paths.
    let stdout = check(&[]);
    let absolute = |file: &str| root.path().join(file).display().to_string();
    assert!(
        stdout.contains(&annotation("warning", &absolute("a.sql"))),
        "{stdout}"
    );
    assert!(!stdout.contains("db1/a.sql"), "{stdout}");
    assert!(
        stdout.contains(&annotation("error", &absolute("db2/schema/a.sql"))),
        "{stdout}"
    );

    // Paths given as arguments are relative to the working directory.
    let stdout = check(&["a.sql", "db1/a.sql", "db2/schema/a.sql"]);
    assert!(stdout.contains(&annotation("warning", "a.sql")), "{stdout}");
    assert!(!stdout.contains("db1/a.sql"), "{stdout}");
    assert!(
        stdout.contains(&annotation("error", "db2/schema/a.sql")),
        "{stdout}"
    );
}

/// Writes `files` below a new temporary directory.
fn write_tree(files: &[(&str, &str)]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let path = root.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    root
}

/// Runs `check` in `cwd` without the connection settings of the environment but with `envs`.
/// Returns whether it succeeded and its output.
fn check_in(cwd: &Path, args: &[&str], envs: &[(&str, &str)]) -> (bool, String) {
    let mut cmd = cargo_bin_cmd!("postgres-language-server");
    for var in [
        "DATABASE_URL",
        "PGHOST",
        "PGPORT",
        "PGUSER",
        "PGPASSWORD",
        "PGDATABASE",
    ] {
        cmd.env_remove(var);
    }
    cmd.env("PGPASSFILE", "/nonexistent/pgpass");
    cmd.envs(envs.iter().copied());
    let output = cmd
        .current_dir(cwd)
        .args(["check", "--log-level", "none"])
        .args(args)
        .output()
        .expect("failed to run CLI");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

/// `check` with paths only loads the configuration files on the way to them and below the folders
/// among them, so a broken configuration file elsewhere doesn't matter.
#[test]
fn check_only_loads_the_configuration_files_of_its_paths() {
    let root = write_tree(&[
        (
            "postgres-language-server.jsonc",
            r#"{ "db": { "disableConnection": true } }"#,
        ),
        (
            "tests/fixtures/broken/postgres-language-server.jsonc",
            "{ this is not json",
        ),
        (
            "db1/postgres-language-server.jsonc",
            r#"{ "db": { "disableConnection": true }, "linter": { "rules": { "banDropColumn": "error" } } }"#,
        ),
        ("a.sql", "select 1;\n"),
        ("db1/schema/a.sql", "alter table users drop column name;\n"),
    ]);

    let (success, output) = check_in(root.path(), &["a.sql"], &[]);
    assert!(success, "{output}");

    for path in ["db1/schema/a.sql", "db1"] {
        let (success, output) = check_in(root.path(), &["--reporter", "github", path], &[]);
        assert!(!success, "{output}");
        assert!(
            output.contains("::error title=lint/banDropColumn"),
            "the configuration of db1 doesn't apply to {path}:\n{output}"
        );
    }
}

/// A nested configuration file that can't be loaded is reported with its path, its folder uses
/// the default settings, and the other files are still checked.
#[test]
fn check_reports_a_broken_nested_configuration_and_continues() {
    let root = write_tree(&[
        (
            "postgres-language-server.jsonc",
            r#"{ "db": { "disableConnection": true }, "linter": { "rules": { "banDropColumn": "error" } } }"#,
        ),
        (
            "tests/fixtures/broken/postgres-language-server.jsonc",
            "{ this is not json",
        ),
        ("a.sql", "alter table users drop column name;\n"),
        (
            "tests/fixtures/broken/b.sql",
            "alter table users drop column name;\n",
        ),
    ]);

    let (success, output) = check_in(root.path(), &[], &[]);
    assert!(!success, "{output}");
    assert!(
        output.lines().any(|line| line.contains("broken")
            && line.contains("postgres-language-server.jsonc")
            && line.contains("configuration")),
        "the broken configuration file is not named:\n{output}"
    );

    let (_, output) = check_in(root.path(), &["--reporter", "github"], &[]);
    let level_of = |file: &str| {
        output
            .lines()
            .find(|line| line.contains("title=lint/banDropColumn") && line.contains(file))
            .and_then(|line| line.strip_prefix("::"))
            .and_then(|line| line.split(' ').next())
    };
    assert_eq!(level_of("a.sql"), Some("error"), "{output}");
    // The defaults, not the root configuration's `error`.
    assert_eq!(level_of("b.sql"), Some("warning"), "{output}");
}

/// The database environment variables don't replace the database of a nested configuration file
/// that sets one, so that `db1/` and `db2/` keep their own databases.
#[cfg(target_os = "linux")]
#[test]
fn the_environment_does_not_replace_the_database_of_a_nested_configuration() {
    let root = write_tree(&[
        (
            "postgres-language-server.jsonc",
            r#"{ "db": { "disableConnection": true } }"#,
        ),
        (
            "db1/postgres-language-server.jsonc",
            r#"{ "db": { "host": "127.0.0.1", "port": 5432, "username": "postgres", "password": "postgres", "database": "nested_db_that_does_not_exist" } }"#,
        ),
        ("db1/a.sql", "select 1;\n"),
    ]);

    for envs in [
        &[][..],
        &[("PGDATABASE", "postgres")][..],
        &[(
            "DATABASE_URL",
            "postgresql://postgres:postgres@127.0.0.1:5432/postgres",
        )][..],
    ] {
        let (_, output) = check_in(root.path(), &["db1/a.sql"], envs);
        assert!(
            output.contains("nested_db_that_does_not_exist"),
            "with {envs:?}, the nested configuration did not use its own database:\n{output}"
        );
    }
}

/// Without a project configuration, `check` says that it uses the user configuration, on stderr
/// so that it stays out of the reporters' output.
#[cfg(unix)]
#[test]
fn check_announces_the_user_configuration() {
    let root = write_tree(&[
        (
            "config/postgres-language-server/postgres-language-server.jsonc",
            r#"{ "db": { "disableConnection": true } }"#,
        ),
        ("project/a.sql", "select 1;\n"),
    ]);
    let config_home = root.path().join("config");
    let output = cargo_bin_cmd!("postgres-language-server")
        .current_dir(root.path().join("project"))
        .env("XDG_CONFIG_HOME", &config_home)
        .args([
            "check",
            "--log-level",
            "none",
            "--reporter",
            "json",
            "a.sql",
        ])
        .output()
        .expect("failed to run CLI");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let user_configuration = config_home
        .join("postgres-language-server")
        .join("postgres-language-server.jsonc");
    assert!(
        stderr.contains(&format!(
            "Using the user configuration at {}",
            user_configuration.display()
        )),
        "{stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("user configuration"),
        "the announcement ended up in the report"
    );
}

/// Relative paths of the configuration file are relative to its folder, also when `check` runs
/// in a folder below it.
#[test]
fn relative_paths_are_relative_to_the_configuration_file() {
    let root = write_tree(&[
        (
            "postgres-language-server.jsonc",
            r#"{ "db": { "disableConnection": true }, "migrations": { "migrationsDir": "migrations", "after": 5 } }"#,
        ),
        (
            "migrations/0009_later.sql",
            "alter table users drop column name;\n",
        ),
        ("sub/a.sql", "select 1;\n"),
    ]);

    let migration = Path::new("..").join("migrations").join("0009_later.sql");
    let (_, output) = check_in(
        &root.path().join("sub"),
        &[migration.to_str().unwrap()],
        &[],
    );
    // `banDropColumn` only runs on migrations once `migrations` is configured.
    assert!(
        output.contains("lint/banDropColumn"),
        "the file is not recognised as a migration:\n{output}"
    );
}

/// A relative path in the arguments is relative to the working directory, like any other path
/// typed on the command line, not to the folder of the configuration file.
#[test]
fn relative_path_arguments_are_relative_to_the_working_directory() {
    let root = write_tree(&[
        (
            "postgres-language-server.jsonc",
            r#"{ "db": { "disableConnection": true } }"#,
        ),
        (
            "sub/migrations/0009_later.sql",
            "alter table users drop column name;\n",
        ),
    ]);

    let migration = Path::new("migrations").join("0009_later.sql");
    let (_, output) = check_in(
        &root.path().join("sub"),
        &[
            "--migrations-dir",
            "migrations",
            "--after",
            "5",
            migration.to_str().unwrap(),
        ],
        &[],
    );
    // `banDropColumn` only runs on migrations once `migrations` is configured.
    assert!(
        output.contains("lint/banDropColumn"),
        "the file is not recognised as a migration:\n{output}"
    );
}

/// Runs `check` with the database settings given in `args` only, without the connection
/// settings of the environment running the tests.
#[cfg(target_os = "linux")]
fn run_check_without_db_env(args: &[&str], envs: &[(&str, &str)]) -> String {
    let mut cmd = cargo_bin_cmd!("postgres-language-server");
    for var in [
        "DATABASE_URL",
        "PGHOST",
        "PGPORT",
        "PGUSER",
        "PGPASSWORD",
        "PGDATABASE",
    ] {
        cmd.env_remove(var);
    }
    // Keep the passfile of the user running the tests out of it.
    cmd.env("PGPASSFILE", "/nonexistent/pgpass");
    cmd.envs(envs.iter().copied());

    let mut full_args = vec!["check", "--config-path", CONFIG_PATH, "--log-level", "none"];
    full_args.extend_from_slice(args);
    let output = cmd.args(full_args).output().expect("failed to run CLI");

    normalize_output(
        output.status,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

fn run_check(args: &[&str]) -> String {
    let mut full_args = vec!["--config-path", CONFIG_PATH, "--log-level", "none"];
    full_args.extend_from_slice(args);
    run_check_with(&full_args, None, None)
}

fn run_check_with(args: &[&str], stdin: Option<&str>, cwd: Option<&Path>) -> String {
    let mut cmd = cargo_bin_cmd!("postgres-language-server");
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    if let Some(input) = stdin {
        cmd.write_stdin(input);
    }

    let mut full_args = vec!["check"];
    full_args.extend_from_slice(args);
    let output = cmd.args(full_args).output().expect("failed to run CLI");

    normalize_output(
        output.status,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

fn normalize_durations(input: &str) -> String {
    let mut content = input.to_owned();

    let mut search_start = 0;
    while let Some(relative) = content[search_start..].find(" in ") {
        let start = search_start + relative + 4;
        // Find end of sentence period: scan for '.' that is followed by
        // a non-digit (or EOL), skipping decimal points inside durations.
        let rest = &content[start..];
        let mut dot_search = 0;
        let mut found_end = None;
        while let Some(dot_rel) = rest[dot_search..].find('.') {
            let dot_pos = dot_search + dot_rel;
            let after_dot = dot_pos + 1;
            if after_dot >= rest.len() || !rest.as_bytes()[after_dot].is_ascii_digit() {
                found_end = Some(start + dot_pos);
                break;
            }
            dot_search = after_dot;
        }
        if let Some(end) = found_end {
            if content[start..end].chars().any(|c| c.is_ascii_digit()) {
                content.replace_range(start..end, "<duration>");
                search_start = start + "<duration>".len() + 1;
                continue;
            }
            search_start = end + 1;
        } else {
            break;
        }
    }

    let mut time_search = 0;
    while let Some(relative) = content[time_search..].find("time=\"") {
        let start = time_search + relative + 6;
        if let Some(end_rel) = content[start..].find('"') {
            let end = start + end_rel;
            if content[start..end].chars().any(|c| c.is_ascii_digit()) {
                content.replace_range(start..end, "<duration>");
            }
            time_search = end + 1;
        } else {
            break;
        }
    }

    // Normalize JSON "duration":12345 and "duration": 12345 (numeric nanos)
    for json_dur_pat in &["\"duration\":", "\"duration\": "] {
        let mut json_search = 0;
        while let Some(relative) = content[json_search..].find(json_dur_pat) {
            let start = json_search + relative + json_dur_pat.len();
            let rest = &content[start..];
            // Skip if it's a string value (already handled or not numeric)
            if rest.starts_with('"') {
                json_search = start + 1;
                continue;
            }
            let num_end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            if num_end > 0 {
                content.replace_range(start..start + num_end, "0");
                json_search = start + 1;
            } else {
                json_search = start + 1;
            }
        }
    }

    // Normalize Rust Debug durations (e.g. "4.877792ms", "1.23s", "123µs", "123ns")
    // used by summary reporter: "file(s) in 4.877ms." / "Completed in 4.877ms."
    for prefix in &["file(s) in ", "Completed in "] {
        let mut search = 0;
        while let Some(relative) = content[search..].find(prefix) {
            let start = search + relative + prefix.len();
            // Find the end of the duration: digits, '.', and time unit suffix
            let rest = &content[start..];
            let dur_end = rest
                .find(|c: char| !c.is_ascii_digit() && c != '.' && c != 'µ')
                .unwrap_or(rest.len());
            // Include trailing unit letters (m, s, n, etc.)
            let after_digits = &rest[dur_end..];
            let unit_end = after_digits
                .find(|c: char| !c.is_ascii_lowercase())
                .unwrap_or(after_digits.len());
            let total_end = start + dur_end + unit_end;
            if total_end > start {
                content.replace_range(start..total_end, "<duration>");
                search = start + "<duration>".len();
            } else {
                search = start + 1;
            }
        }
    }

    content
}

fn normalize_output(status: ExitStatus, stdout: &str, stderr: &str) -> String {
    let normalized_stdout = normalize_durations(stdout);
    let normalized_stderr = normalize_diagnostics(stderr);
    let status_label = if status.success() {
        "success"
    } else {
        "failure"
    };
    format!(
        "status: {status_label}\nstdout:\n{}\nstderr:\n{}\n",
        normalized_stdout.trim_end(),
        normalized_stderr.trim_end()
    )
}

fn normalize_diagnostics(input: &str) -> String {
    let normalized = normalize_durations(input);
    let mut lines = normalized.lines().peekable();
    let mut diagnostic_sections: Vec<String> = Vec::new();
    let mut other_lines: Vec<String> = Vec::new();

    while let Some(line) = lines.next() {
        if is_path_line(line) {
            let mut block = String::from(line);
            while let Some(&next) = lines.peek() {
                if is_path_line(next) || next.starts_with("check ") {
                    break;
                }
                block.push('\n');
                block.push_str(next);
                lines.next();
            }
            diagnostic_sections.push(trim_trailing_newlines(block));
        } else {
            other_lines.push(line.to_string());
        }
    }

    diagnostic_sections.sort();

    let mut parts = Vec::new();
    if !diagnostic_sections.is_empty() {
        parts.push(diagnostic_sections.join("\n\n"));
    }

    let rest = trim_trailing_newlines(other_lines.join("\n"));
    if rest.trim().is_empty() {
        parts.join("\n\n")
    } else if parts.is_empty() {
        rest
    } else {
        parts.push(rest);
        parts.join("\n\n")
    }
}

fn is_path_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    (trimmed.starts_with("./") || trimmed.starts_with("tests/"))
        && trimmed.contains(':')
        && trimmed.contains(" syntax")
}

fn trim_trailing_newlines(mut value: String) -> String {
    while value.ends_with('\n') {
        value.pop();
    }
    value
}

/// Runs `check` with the given arguments on a file with `content`, with the test config plus the
/// `config` entries, and returns the file afterwards and the output of the command.
fn check_file(name: &str, config: &str, args: &[&str], content: &str) -> (String, String) {
    let dir = Path::new("tests/fixtures").join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut base = std::fs::read_to_string(CONFIG_PATH).unwrap();
    if !config.is_empty() {
        base = format!("{},{config}}}", base.trim_end().strip_suffix('}').unwrap());
    }
    let config_path = dir.join("postgres-language-server.jsonc");
    std::fs::write(&config_path, base).unwrap();
    let file = dir.join("migration.sql");
    std::fs::write(&file, content).unwrap();

    let mut cmd = cargo_bin_cmd!("postgres-language-server");
    let output = cmd
        .args(["check", "--config-path", config_path.to_str().unwrap()])
        .args(["--log-level", "none", "--disable-db"])
        .args(args)
        .arg(file.to_str().unwrap())
        .output()
        .expect("failed to run CLI");

    let fixed = std::fs::read_to_string(&file).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (fixed, output)
}

/// The file after `check` with the given arguments, see [`check_file`].
fn check_fixed(name: &str, config: &str, args: &[&str], content: &str) -> String {
    check_file(name, config, args, content).0
}

#[test]
fn check_fix_writes_safe_fixes() {
    // The section runs outside a transaction, so adding `CONCURRENTLY` is safe.
    let content = "-- migrate:up transaction:false\nreindex index my_index;\n";
    assert_eq!(
        check_fixed(
            "fix-safe",
            r#""migrations": {"sectionMarkers": "dbmate"}"#,
            &["--fix"],
            content
        ),
        "-- migrate:up transaction:false\nreindex index concurrently my_index;\n",
    );
}

#[test]
fn check_fix_without_section_markers_needs_unsafe() {
    // Without section markers, we don't know whether the migration tool runs the file in a
    // transaction, where `CONCURRENTLY` fails.
    let content = "reindex index my_index;\n";
    let (fixed, output) = check_file("fix-plain", "", &["--fix"], content);
    assert_eq!(fixed, content);
    assert!(
        output.contains("Skipped 1 unsafe fix. Rerun with --fix --unsafe to apply them."),
        "{output}"
    );
    assert_eq!(
        check_fixed("fix-plain-unsafe", "", &["--fix", "--unsafe"], content),
        "reindex index concurrently my_index;\n"
    );
}

#[test]
fn check_reports_fixable_diagnostics() {
    let (_, output) = check_file("fixable", "", &[], "reindex index my_index;\n");
    assert!(output.contains(" FIXABLE "), "{output}");
    assert!(
        output.contains("1 diagnostic has a fix. Run with --fix to apply the safe fixes, and with --fix --unsafe to apply all of them."),
        "{output}"
    );
    assert!(!output.contains("No fixes applied."), "{output}");
}

#[test]
fn check_fix_only_applies_the_selected_rules() {
    let content = "create table t (a json, b varchar(10));\n";
    let args = ["--fix", "--unsafe", "--only=preferJsonb"];
    let (fixed, output) = check_file("fix-only", "", &args, content);
    assert_eq!(
        fixed, "create table t (a jsonb, b varchar(10));\n",
        "{output}"
    );
    // The diagnostics of other rules are left out too.
    assert!(!output.contains("preferTextField"), "{output}");

    let args = ["--fix", "--unsafe", "--only=style", "--skip=preferJsonb"];
    assert_eq!(
        check_fixed("fix-skip", "", &args, content),
        "create table t (a json, b text);\n"
    );
}

#[test]
fn check_stdin_applies_fixes() {
    let output = run_check_with(
        &[
            "--config-path",
            CONFIG_PATH,
            "--log-level",
            "none",
            "--fix",
            "--unsafe",
            "--stdin-file-path",
            "tests/fixtures/virtual.sql",
        ],
        Some("reindex index my_index;\n"),
        None,
    );
    assert!(
        output.contains("stdout:\nreindex index concurrently my_index;\n"),
        "{output}"
    );
}
