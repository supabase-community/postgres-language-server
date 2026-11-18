//! Records the regression fixtures of one Postgres major version.
//!
//! Usage: `record <major> [tag] [--catalog-only]`, normally through `just record-regress`. Without
//! a tag, the one in `data/<major>/SOURCE` is used.
//!
//! Fetches `src/test/regress/sql` of the tag, copies the files verbatim, runs every statement
//! against `postgres:<major>.<minor>` in Docker and writes Postgres' verdict per statement. It also
//! writes the catalog of a fresh database. With `--catalog-only`, only the catalog is re-recorded.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use pgls_postgres_regress::{CATALOG_FILE, Statement, Verdict, data_dir, preprocess, split};
use sqlx::{Connection, Executor, PgConnection};

const PASSWORD: &str = "postgres";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let catalog_only = args.iter().any(|x| x == "--catalog-only");
    args.retain(|x| x != "--catalog-only");
    let (major, tag) = match args.as_slice() {
        [major] => (major, None),
        [major, tag] => (major, Some(tag.clone())),
        _ => fail("usage: record <major> [tag] [--catalog-only]"),
    };
    let major: u32 = major
        .parse()
        .unwrap_or_else(|_| fail(&format!("not a major version: {major}")));
    let dir = data_dir().join(major.to_string());
    let tag = tag.unwrap_or_else(|| {
        fs::read_to_string(dir.join("SOURCE"))
            .unwrap_or_else(|_| fail(&format!("no tag given and no {major}/SOURCE yet")))
            .trim()
            .to_owned()
    });
    let version = tag
        .strip_prefix(&format!("REL_{major}_"))
        .filter(|minor| !minor.is_empty() && minor.chars().all(|c| c.is_ascii_digit()))
        .map(|minor| format!("{major}.{minor}"))
        .unwrap_or_else(|| {
            fail(&format!(
                "expected a tag like REL_{major}_<minor>, got {tag}"
            ))
        });

    if !catalog_only {
        copy_upstream(&tag, &dir)?;
    }

    let container = Container::start(&format!("postgres:{version}"))?;
    let ctrl_c_id = container.id.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        remove_container(&ctrl_c_id);
        std::process::exit(130);
    });
    let base_url = format!(
        "postgresql://postgres:{PASSWORD}@127.0.0.1:{}",
        container.port
    );
    let mut admin = connect_with_retry(&format!("{base_url}/postgres")).await?;
    let server_version: String = sqlx::query_scalar("SHOW server_version")
        .fetch_one(&mut admin)
        .await
        .map_err(|e| format!("query server version: {e}"))?;
    let matches = server_version
        .strip_prefix(&version)
        .is_some_and(|rest| !rest.starts_with(|c: char| c.is_ascii_digit() || c == '.'));
    if !matches {
        return Err(format!("server version {server_version} doesn't match {tag}").into());
    }

    // Before the regression files run, the `postgres` database is still fresh.
    record_catalog(&format!("{base_url}/postgres"), &dir).await?;
    if catalog_only {
        return Ok(());
    }

    let sql_dir = dir.join("sql");
    let verdicts_dir = dir.join("verdicts");
    fs::create_dir_all(&verdicts_dir).map_err(|e| format!("create verdicts directory: {e}"))?;
    let files = sql_files(&sql_dir)?;
    let mut totals = Totals::default();
    for (file_no, path) in files.iter().enumerate() {
        let name = path
            .file_stem()
            .ok_or_else(|| format!("no file name: {}", path.display()))?
            .to_string_lossy();
        let source = preprocess(
            &fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?,
        );
        let statements = record_file(&mut admin, &base_url, file_no, &source).await?;
        let counts = Totals::of(&statements);
        eprintln!(
            "[{}/{}] {name}: {} accepted, {} rejected, {} skipped",
            file_no + 1,
            files.len(),
            counts.accepted,
            counts.rejected,
            counts.skipped
        );
        totals.add(&counts);
        fs::write(
            verdicts_dir.join(format!("{name}.txt")),
            statements
                .iter()
                .map(|statement| format!("{statement}\n"))
                .collect::<String>(),
        )
        .map_err(|e| format!("write verdict file of {name}: {e}"))?;
    }
    eprintln!(
        "Postgres {server_version}: {} accepted, {} rejected, {} skipped",
        totals.accepted, totals.rejected, totals.skipped
    );
    Ok(())
}

/// Writes the catalog of the database at `url` as gzipped `pgls_catalog::Snapshot` JSON.
async fn record_catalog(url: &str, dir: &Path) -> Result<()> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await
        .map_err(|e| format!("connect to Postgres: {e}"))?;
    let snapshot = pgls_catalog::Snapshot::load(&pool)
        .await
        .map_err(|e| format!("load the catalog: {e}"))?;
    pool.close().await;
    let file = fs::File::create(dir.join(CATALOG_FILE))
        .map_err(|e| format!("create catalog file: {e}"))?;
    let mut gz = flate2::write::GzEncoder::new(file, flate2::Compression::best());
    serde_json::to_writer(&mut gz, &snapshot).map_err(|e| format!("write catalog: {e}"))?;
    gz.finish()
        .map_err(|e| format!("finish catalog file: {e}"))?;
    eprintln!(
        "Recorded the catalog: {} types, {} functions, {} operators, {} casts",
        snapshot.types.len(),
        snapshot.functions.len(),
        snapshot.operators.len(),
        snapshot.casts.len()
    );
    Ok(())
}

/// Replaces `data/<major>` with the regression SQL of `tag`.
fn copy_upstream(tag: &str, dir: &Path) -> Result<()> {
    let checkout = std::env::temp_dir().join(format!("pgls-regress-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&checkout);
    eprintln!("Fetching src/test/regress/sql of {tag}");
    run(Command::new("git")
        .args([
            "-c",
            "advice.detachedHead=false",
            "clone",
            "--quiet",
            "--depth",
            "1",
        ])
        .args(["--branch", tag, "--filter=blob:none", "--sparse"])
        .arg("https://github.com/postgres/postgres")
        .arg(&checkout))?;
    run(Command::new("git").arg("-C").arg(&checkout).args([
        "sparse-checkout",
        "set",
        "src/test/regress/sql",
    ]))?;

    for sub in ["sql", "verdicts"] {
        let _ = fs::remove_dir_all(dir.join(sub));
    }
    let sql_dir = dir.join("sql");
    fs::create_dir_all(&sql_dir).map_err(|e| format!("create sql directory: {e}"))?;
    for path in sql_files(&checkout.join("src/test/regress/sql"))? {
        let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let name = path
            .file_name()
            .ok_or_else(|| format!("no file name: {}", path.display()))?;
        // The loader works on text; such files only matter on Windows anyway.
        if std::str::from_utf8(&bytes).is_err() {
            eprintln!("Leaving out {}: not UTF-8", name.to_string_lossy());
            continue;
        }
        fs::write(sql_dir.join(name), bytes).map_err(|e| format!("write SQL file: {e}"))?;
    }
    fs::write(dir.join("SOURCE"), format!("{tag}\n")).map_err(|e| format!("write SOURCE: {e}"))?;
    let _ = fs::remove_dir_all(&checkout);
    Ok(())
}

/// Runs the statements of one file in a fresh database.
async fn record_file(
    admin: &mut PgConnection,
    base_url: &str,
    file_no: usize,
    source: &str,
) -> Result<Vec<Statement>> {
    let db = format!("regress_{file_no}");
    // A database left over from an interrupted run.
    sqlx::raw_sql(&format!("DROP DATABASE IF EXISTS {db} WITH (FORCE)"))
        .execute(&mut *admin)
        .await
        .map_err(|e| format!("drop {db}: {e}"))?;
    sqlx::raw_sql(&format!("CREATE DATABASE {db}"))
        .execute(&mut *admin)
        .await
        .map_err(|e| format!("create {db}: {e}"))?;
    let url = format!("{base_url}/{db}");
    let mut conn = PgConnection::connect(&url)
        .await
        .map_err(|e| format!("connect to {db}: {e}"))?;
    conn.execute("SET statement_timeout = '5s'")
        .await
        .map_err(|e| format!("set statement timeout: {e}"))?;

    let mut statements = Vec::new();
    // Set when a statement hangs on the client side; the connection is unusable from then on.
    let mut stuck = false;
    for range in split(source) {
        let sql = &source[range];
        let verdict = if stuck || transaction_control(sql) || copy_with_client(sql) {
            Verdict::Skipped
        } else {
            // Not a prepared statement: the sqlx statement cache masks errors.
            let execute = sqlx::raw_sql(sql).execute(&mut conn);
            match tokio::time::timeout(Duration::from_secs(30), execute).await {
                Err(_) => {
                    stuck = true;
                    Verdict::Skipped
                }
                Ok(Ok(_)) => Verdict::Accepted,
                Ok(Err(e)) => match e.as_database_error().and_then(|x| x.code()).as_deref() {
                    // query_canceled, i.e. statement_timeout
                    Some("57014") => Verdict::Skipped,
                    Some(state) => Verdict::Rejected(state.to_owned()),
                    None => Verdict::Rejected("CLIENT_ERROR".into()),
                },
            }
        };
        statements.push(Statement { range, verdict });
    }
    let _ = tokio::time::timeout(Duration::from_secs(5), conn.close()).await;

    // Roles are shared by all databases.
    let roles = regress_roles(admin).await?;
    if !roles.is_empty()
        && let Ok(mut conn) = PgConnection::connect(&url).await
    {
        drop_owned(&mut conn, &roles).await;
        conn.close().await.ok();
    }
    sqlx::raw_sql(&format!("DROP DATABASE {db} WITH (FORCE)"))
        .execute(&mut *admin)
        .await
        .map_err(|e| format!("drop {db}: {e}"))?;
    drop_owned(admin, &roles).await;
    for role in &roles {
        sqlx::raw_sql(&format!("DROP ROLE IF EXISTS {role}"))
            .execute(&mut *admin)
            .await
            .ok();
    }
    Ok(statements)
}

/// The roles the regression tests create; they are all named `regress_*`.
async fn regress_roles(conn: &mut PgConnection) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar(
        "select quote_ident(rolname) from pg_roles where rolname like 'regress\\_%'",
    )
    .fetch_all(conn)
    .await
    .map_err(|e| format!("list regression roles: {e}"))?)
}

async fn drop_owned(conn: &mut PgConnection, roles: &[String]) {
    for role in roles {
        sqlx::raw_sql(&format!("DROP OWNED BY {role} CASCADE"))
            .execute(&mut *conn)
            .await
            .ok();
    }
}

/// `COPY` to or from the client. Postgres also accepts `FROM STDOUT` and `TO STDIN`.
fn copy_with_client(sql: &str) -> bool {
    let upper = sql.to_ascii_uppercase();
    upper.starts_with("COPY ") && (upper.contains("STDIN") || upper.contains("STDOUT"))
}

fn transaction_control(sql: &str) -> bool {
    let normalized = sql.to_ascii_lowercase();
    [
        "begin",
        "start transaction",
        "commit",
        "end",
        "rollback",
        "savepoint",
        "release savepoint",
        "prepare transaction",
    ]
    .iter()
    .any(|x| normalized.starts_with(x))
}

fn sql_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
        let path = entry
            .map_err(|e| format!("read an entry of {}: {e}", dir.display()))?
            .path();
        if path.extension().is_some_and(|x| x == "sql") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

struct Container {
    id: String,
    port: String,
}

impl Container {
    fn start(image: &str) -> Result<Self> {
        eprintln!("Starting {image}");
        let id = output(
            Command::new("docker")
                .args(["run", "--detach", "--rm", "--publish", "127.0.0.1::5432"])
                .args(["--env", &format!("POSTGRES_PASSWORD={PASSWORD}")])
                .arg(image),
        )?;
        // Removes the container if anything below fails.
        let mut container = Container {
            id,
            port: String::new(),
        };
        let address = output(Command::new("docker").args(["port", &container.id, "5432/tcp"]))?;
        container.port = address
            .lines()
            .next()
            .and_then(|x| x.rsplit_once(':'))
            .map(|(_, port)| port.to_owned())
            .ok_or_else(|| format!("unexpected `docker port` output: {address}"))?;
        Ok(container)
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        remove_container(&self.id);
    }
}

fn remove_container(id: &str) {
    let _ = Command::new("docker")
        .args(["rm", "--force", id])
        .stdout(Stdio::null())
        .status();
}

async fn connect_with_retry(url: &str) -> Result<PgConnection> {
    let mut attempts = 0;
    loop {
        match PgConnection::connect(url).await {
            Ok(conn) => return Ok(conn),
            Err(e) if attempts < 120 => {
                attempts += 1;
                if attempts % 20 == 0 {
                    eprintln!("Waiting for Postgres: {e}");
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => return Err(format!("connect to Postgres: {e}").into()),
        }
    }
}

#[derive(Default)]
struct Totals {
    accepted: usize,
    rejected: usize,
    skipped: usize,
}

impl Totals {
    fn of(statements: &[Statement]) -> Self {
        let mut totals = Totals::default();
        for statement in statements {
            match statement.verdict {
                Verdict::Accepted => totals.accepted += 1,
                Verdict::Rejected(_) => totals.rejected += 1,
                Verdict::Skipped => totals.skipped += 1,
            }
        }
        totals
    }

    fn add(&mut self, other: &Totals) {
        self.accepted += other.accepted;
        self.rejected += other.rejected;
        self.skipped += other.skipped;
    }
}

fn run(command: &mut Command) -> Result<()> {
    let status = command
        .status()
        .map_err(|e| format!("spawn {command:?}: {e}"))?;
    if !status.success() {
        return Err(format!("{command:?} failed with {status}").into());
    }
    Ok(())
}

fn output(command: &mut Command) -> Result<String> {
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("spawn {command:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{command:?} failed with {}", output.status).into());
    }
    Ok(String::from_utf8(output.stdout)
        .map_err(|e| format!("output of {command:?} isn't UTF-8: {e}"))?
        .trim()
        .to_owned())
}

/// Exits on a usage error. Once the container runs, errors are returned instead, so it gets removed.
fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1)
}
