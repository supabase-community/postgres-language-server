use bpaf::Args;
use std::path::Path;

use crate::run_cli;
use pgls_console::BufferConsole;
use pgls_fs::MemoryFileSystem;
use pgls_workspace::DynRef;

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
