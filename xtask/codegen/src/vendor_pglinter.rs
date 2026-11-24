use crate::glue::{fs2, pushd, rm_rf};
use crate::{ensure, project_root, run, Result};

const REPOSITORY: &str = "https://github.com/pmpetit/pglinter.git";

/// Copies pglinter's `sql/rules.sql` at `commit`, or at the commit in
/// `crates/pgls_pglinter/vendor/COMMIT_SHA.txt` when it's `None`, into
/// `crates/pgls_pglinter/vendor/`, and pins that commit. `xtask_codegen -- pglinter` generates the
/// rules from it.
pub fn vendor_pglinter(commit: Option<String>) -> Result<()> {
    let vendor_dir = project_root().join("crates/pgls_pglinter/vendor");
    let commit_file = vendor_dir.join("COMMIT_SHA.txt");
    let commit = match commit {
        Some(commit) => commit,
        None => fs2::read_to_string(&commit_file)?.trim().to_owned(),
    };
    ensure!(
        commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "the pglinter commit must be a full SHA, got `{commit}`"
    );

    let checkout = project_root().join("target/vendor/pglinter");
    rm_rf(&checkout)?;
    fs2::create_dir_all(&checkout)?;
    {
        let _dir = pushd(&checkout)?;
        run!("git init --quiet")?;
        run!("git fetch --quiet --depth 1 {REPOSITORY} {commit}")?;
        run!("git checkout --quiet FETCH_HEAD")?;
    }

    fs2::create_dir_all(vendor_dir.join("sql"))?;
    fs2::copy(
        checkout.join("sql/rules.sql"),
        vendor_dir.join("sql/rules.sql"),
    )?;
    fs2::write(&commit_file, &commit)?;

    rm_rf(&checkout)
}
