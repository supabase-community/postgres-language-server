use bpaf::Args;
use std::path::Path;

use crate::run_cli;
use pgt_console::BufferConsole;
use pgt_fs::MemoryFileSystem;
use pgt_workspace::DynRef;

#[test]
fn syntax_error() {
    let mut fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file_path = Path::new("test.sql");
    fs.insert(file_path.into(), "select 1".as_bytes());

    let result = run_cli(
        DynRef::Borrowed(&mut fs),
        &mut console,
        Args::from([("check"), file_path.as_os_str().to_str().unwrap()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");
}

fn sample_connection_string() -> &'static str {
    "postgres://postgres:postgres@localhost:5432/postgres"
}

#[test]
fn accepts_connection_string_from_config() {
    let mut fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file_path = Path::new("test.sql");
    fs.insert(file_path.into(), "select 1".as_bytes());

    fs.insert(
        Path::new("postgres-language-server.jsonc").into(),
        format!(
            "{{\n  \"db\": {{ \"connectionString\": \"{}\" }},\n  \"typecheck\": {{ \"enabled\": false }},\n  \"plpgsqlCheck\": {{ \"enabled\": false }}\n}}",
            sample_connection_string()
        )
        .into_bytes(),
    );

    let result = run_cli(
        DynRef::Borrowed(&mut fs),
        &mut console,
        Args::from(["check", file_path.as_os_str().to_str().unwrap()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");
}

#[test]
fn accepts_connection_string_flag() {
    let mut fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file_path = Path::new("test.sql");
    fs.insert(file_path.into(), "select 1".as_bytes());

    fs.insert(
        Path::new("postgres-language-server.jsonc").into(),
        b"{\n  \"typecheck\": { \"enabled\": false },\n  \"plpgsqlCheck\": { \"enabled\": false }\n}\n".to_vec(),
    );

    let result = run_cli(
        DynRef::Borrowed(&mut fs),
        &mut console,
        Args::from(
            [
                "check",
                "--connection-string",
                sample_connection_string(),
                file_path.as_os_str().to_str().unwrap(),
            ]
            .as_slice(),
        ),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");
}
