//! Records the regression fixtures of one Postgres major version.
//!
//! Usage: `record <major> [tag] [--catalog-only]`, normally through `just record-regress`. Without
//! a tag, the one in `data/<major>/SOURCE` is used.
//!
//! Fetches `src/test/regress/sql` of the tag, copies the files verbatim, runs every statement
//! against `postgres:<major>.<minor>` in Docker and writes Postgres' verdict per statement, and the
//! types of the columns of the statements that return rows. It also writes the catalog of a fresh
//! database. With `--catalog-only`, only the catalog is re-recorded.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use pgls_postgres_regress::{
    CATALOG_FILE, OUTPUT_TYPES_DIR, Statement, VERDICTS_DIR, Verdict, data_dir, preprocess, split,
};
use sqlx::{
    Column, ConnectOptions, Connection, Executor, PgConnection, Row, Statement as _,
    postgres::PgConnectOptions,
};

const PASSWORD: &str = "postgres";

/// How long the client waits for an answer from the server. `statement_timeout` cancels a
/// statement after 5 seconds, so a statement that takes longer hangs on the client side.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
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
        copy_upstream(&tag, &dir);
    }

    let container = Container::start(&format!("postgres:{version}"));
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
    let mut admin = connect_with_retry(&format!("{base_url}/postgres")).await;
    let server_version: String = sqlx::query_scalar("SHOW server_version")
        .fetch_one(&mut admin)
        .await
        .expect("query server version");
    let matches = server_version
        .strip_prefix(&version)
        .is_some_and(|rest| !rest.starts_with(|c: char| c.is_ascii_digit() || c == '.'));
    assert!(
        matches,
        "server version {server_version} doesn't match {tag}"
    );

    // Before the regression files run, the `postgres` database is still fresh.
    record_catalog(&format!("{base_url}/postgres"), &dir).await;
    if catalog_only {
        return;
    }

    let sql_dir = dir.join("sql");
    let verdicts_dir = dir.join(VERDICTS_DIR);
    let output_types_dir = dir.join(OUTPUT_TYPES_DIR);
    for sub in [&verdicts_dir, &output_types_dir] {
        fs::create_dir_all(sub).expect("create fixture directory");
    }
    let builtin_types = TypeNames::fetch(&mut admin, &format!("t.oid < {FIRST_NORMAL_OBJECT_ID}"))
        .await
        .expect("look up the built-in types");
    let files = sql_files(&sql_dir);
    let mut totals = Totals::default();
    for (file_no, path) in files.iter().enumerate() {
        let name = path.file_stem().unwrap().to_string_lossy();
        let source = preprocess(&fs::read_to_string(path).expect("read SQL file"));
        let statements = record_file(&mut admin, &base_url, file_no, &source, &builtin_types).await;
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
        .expect("write verdict file");
        fs::write(
            output_types_dir.join(format!("{name}.txt")),
            statements
                .iter()
                .filter_map(Statement::output_types_line)
                .map(|line| format!("{line}\n"))
                .collect::<String>(),
        )
        .expect("write output types file");
    }
    eprintln!(
        "Postgres {server_version}: {} accepted, {} rejected, {} skipped",
        totals.accepted, totals.rejected, totals.skipped
    );
}

/// Writes the catalog of the database at `url` as gzipped `pgls_catalog::Snapshot` JSON.
async fn record_catalog(url: &str, dir: &Path) {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await
        .expect("connect to Postgres");
    let snapshot = pgls_catalog::Snapshot::load(&pool)
        .await
        .expect("load the catalog");
    pool.close().await;
    let file = fs::File::create(dir.join(CATALOG_FILE)).expect("create catalog file");
    let mut gz = flate2::write::GzEncoder::new(file, flate2::Compression::best());
    serde_json::to_writer(&mut gz, &snapshot).expect("write catalog");
    gz.finish().expect("finish catalog file");
    eprintln!(
        "Recorded the catalog: {} types, {} functions, {} operators, {} casts",
        snapshot.types.len(),
        snapshot.functions.len(),
        snapshot.operators.len(),
        snapshot.casts.len()
    );
}

/// Replaces `data/<major>` with the regression SQL of `tag`.
fn copy_upstream(tag: &str, dir: &Path) {
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
        .arg(&checkout));
    run(Command::new("git").arg("-C").arg(&checkout).args([
        "sparse-checkout",
        "set",
        "src/test/regress/sql",
    ]));

    for sub in ["sql", VERDICTS_DIR, OUTPUT_TYPES_DIR] {
        let _ = fs::remove_dir_all(dir.join(sub));
    }
    let sql_dir = dir.join("sql");
    fs::create_dir_all(&sql_dir).expect("create sql directory");
    for path in sql_files(&checkout.join("src/test/regress/sql")) {
        let bytes = fs::read(&path).expect("read upstream SQL file");
        let name = path.file_name().unwrap();
        // The loader works on text; such files only matter on Windows anyway.
        if std::str::from_utf8(&bytes).is_err() {
            eprintln!("Leaving out {}: not UTF-8", name.to_string_lossy());
            continue;
        }
        fs::write(sql_dir.join(name), bytes).expect("write SQL file");
    }
    fs::write(dir.join("SOURCE"), format!("{tag}\n")).expect("write SOURCE");
    let _ = fs::remove_dir_all(&checkout);
}

/// Runs the statements of one file in a fresh database.
async fn record_file(
    admin: &mut PgConnection,
    base_url: &str,
    file_no: usize,
    source: &str,
    builtin_types: &TypeNames,
) -> Vec<Statement> {
    let db = format!("regress_{file_no}");
    // A database left over from an interrupted run.
    sqlx::raw_sql(&format!("DROP DATABASE IF EXISTS {db} WITH (FORCE)"))
        .execute(&mut *admin)
        .await
        .unwrap_or_else(|e| panic!("drop {db}: {e}"));
    sqlx::raw_sql(&format!("CREATE DATABASE {db}"))
        .execute(&mut *admin)
        .await
        .unwrap_or_else(|e| panic!("create {db}: {e}"));
    let url = format!("{base_url}/{db}");
    let mut client = Client::connect(&url).await;
    let mut statements = Vec::new();
    for range in split(source) {
        let sql = &source[range];
        let (verdict, output_types) =
            if client.stuck || transaction_control(sql) || copy_with_client(sql) {
                (Verdict::Skipped, None)
            } else {
                let output_types = client.output_types(sql, builtin_types).await;
                (client.verdict(sql).await, output_types)
            };
        statements.push(Statement {
            range,
            output_types: output_types.filter(|_| verdict == Verdict::Accepted),
            verdict,
        });
    }
    let _ = tokio::time::timeout(Duration::from_secs(5), client.conn.close()).await;

    // Roles are shared by all databases.
    let roles = regress_roles(admin).await;
    if !roles.is_empty()
        && let Ok(mut conn) = PgConnection::connect(&url).await
    {
        drop_owned(&mut conn, &roles).await;
        conn.close().await.ok();
    }
    sqlx::raw_sql(&format!("DROP DATABASE {db} WITH (FORCE)"))
        .execute(&mut *admin)
        .await
        .unwrap_or_else(|e| panic!("drop {db}: {e}"));
    drop_owned(admin, &roles).await;
    for role in &roles {
        sqlx::raw_sql(&format!("DROP ROLE IF EXISTS {role}"))
            .execute(&mut *admin)
            .await
            .ok();
    }
    statements
}

/// The connection that runs the statements of a file.
struct Client {
    conn: PgConnection,
    /// Set when a statement hangs on the client side; the connection is unusable from then on.
    stuck: bool,
}

impl Client {
    async fn connect(url: &str) -> Self {
        let options = url
            .parse::<PgConnectOptions>()
            .unwrap_or_else(|e| panic!("parse {url}: {e}"))
            // A cached statement would keep the columns it had before the file changed the tables
            // it reads.
            .statement_cache_capacity(0);
        let mut conn = options
            .connect()
            .await
            .unwrap_or_else(|e| panic!("connect to {url}: {e}"));
        conn.execute("SET statement_timeout = '5s'")
            .await
            .expect("set statement timeout");
        Client { conn, stuck: false }
    }

    /// The types of the columns of the rows that a statement returns, as Postgres describes
    /// them to a client before the statement runs, as `schema.name`. Postgres describes a column
    /// of a domain by the domain's base type ([`SendRowDescriptionMessage`]).
    ///
    /// `None` if the statement returns no rows or no columns, or Postgres can't describe it, e.g.
    /// because it rejects it.
    ///
    /// Preparing a statement only parses and analyses it, and the session keeps the prepared
    /// statement, which only shows in `pg_prepared_statements`.
    ///
    /// [`SendRowDescriptionMessage`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/common/printtup.c#L166
    async fn output_types(&mut self, sql: &str, builtin: &TypeNames) -> Option<Vec<String>> {
        let oids = match tokio::time::timeout(CLIENT_TIMEOUT, self.conn.prepare(sql)).await {
            Err(_) => {
                self.stuck = true;
                return None;
            }
            Ok(prepared) => prepared
                .ok()?
                .columns()
                .iter()
                .map(|column| Some(column.type_info().oid()?.0))
                .collect::<Option<Vec<_>>>()?,
        };
        if oids.is_empty() {
            return None;
        }
        let created = oids
            .iter()
            .filter(|oid| !builtin.0.contains_key(oid))
            .map(u32::to_string)
            .collect::<Vec<_>>();
        let created = if created.is_empty() {
            TypeNames::default()
        } else {
            TypeNames::fetch(
                &mut self.conn,
                &format!("t.oid in ({})", created.join(", ")),
            )
            .await?
        };
        oids.iter()
            .map(|oid| builtin.0.get(oid).or_else(|| created.0.get(oid)).cloned())
            .collect()
    }

    /// Runs a statement in the simple query protocol, like psql.
    async fn verdict(&mut self, sql: &str) -> Verdict {
        if self.stuck {
            return Verdict::Skipped;
        }
        let execute = sqlx::raw_sql(sql).execute(&mut self.conn);
        match tokio::time::timeout(CLIENT_TIMEOUT, execute).await {
            Err(_) => {
                self.stuck = true;
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
    }
}

/// The oid of the first object that isn't built in ([`FirstNormalObjectId`]).
///
/// [`FirstNormalObjectId`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/access/transam.h#L197
const FIRST_NORMAL_OBJECT_ID: u32 = 16384;

/// Names of types by oid, as `schema.name`, with `pg_temp` for the session's temporary schema.
#[derive(Default)]
struct TypeNames(HashMap<u32, String>);

impl TypeNames {
    /// The names of the types that match `filter`, a condition on `pg_type t`.
    async fn fetch(conn: &mut PgConnection, filter: &str) -> Option<Self> {
        let query = format!(
            "select t.oid::int8, \
                 case when n.oid = pg_catalog.pg_my_temp_schema() then 'pg_temp' \
                 else n.nspname::text end || '.' || t.typname::text \
             from pg_catalog.pg_type t \
             join pg_catalog.pg_namespace n on n.oid = t.typnamespace \
             where {filter}"
        );
        let rows = tokio::time::timeout(CLIENT_TIMEOUT, sqlx::raw_sql(&query).fetch_all(conn))
            .await
            .ok()?
            .ok()?;
        rows.iter()
            .map(|row| {
                let oid = u32::try_from(row.try_get::<i64, _>(0).ok()?).ok()?;
                Some((oid, row.try_get::<String, _>(1).ok()?))
            })
            .collect::<Option<_>>()
            .map(TypeNames)
    }
}

/// The roles the regression tests create; they are all named `regress_*`.
async fn regress_roles(conn: &mut PgConnection) -> Vec<String> {
    sqlx::query_scalar("select quote_ident(rolname) from pg_roles where rolname like 'regress\\_%'")
        .fetch_all(conn)
        .await
        .expect("list regression roles")
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

fn sql_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("read directory entry").path())
        .filter(|path| path.extension().is_some_and(|x| x == "sql"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

struct Container {
    id: String,
    port: String,
}

impl Container {
    fn start(image: &str) -> Self {
        eprintln!("Starting {image}");
        let id = output(
            Command::new("docker")
                .args(["run", "--detach", "--rm", "--publish", "127.0.0.1::5432"])
                .args(["--env", &format!("POSTGRES_PASSWORD={PASSWORD}")])
                .arg(image),
        );
        // Removes the container if anything below fails.
        let mut container = Container {
            id,
            port: String::new(),
        };
        let address = output(Command::new("docker").args(["port", &container.id, "5432/tcp"]));
        container.port = address
            .lines()
            .next()
            .and_then(|x| x.rsplit_once(':'))
            .map(|(_, port)| port.to_owned())
            .unwrap_or_else(|| panic!("unexpected `docker port` output: {address}"));
        container
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

async fn connect_with_retry(url: &str) -> PgConnection {
    let mut attempts = 0;
    loop {
        match PgConnection::connect(url).await {
            Ok(conn) => return conn,
            Err(e) if attempts < 120 => {
                attempts += 1;
                if attempts % 20 == 0 {
                    eprintln!("Waiting for Postgres: {e}");
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => panic!("connect to Postgres: {e}"),
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

fn run(command: &mut Command) {
    let status = command.status().expect("spawn command");
    assert!(status.success(), "{command:?} failed with {status}");
}

fn output(command: &mut Command) -> String {
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .expect("spawn command");
    assert!(
        output.status.success(),
        "{command:?} failed with {}",
        output.status
    );
    String::from_utf8(output.stdout)
        .expect("command output is UTF-8")
        .trim()
        .to_owned()
}

/// Exits on a usage error. Once the container runs, errors panic instead, so it gets removed.
fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1)
}
