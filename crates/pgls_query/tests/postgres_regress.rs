//! Every statement Postgres accepted in its own regression suite must parse.

use pgls_postgres_regress::Verdict;

#[test]
fn parses_every_statement_postgres_accepted() {
    let parser_major = pgls_query::parse("select 1")
        .expect("parse select 1")
        .protobuf
        .version as u32
        / 10000;
    let mut checked = 0;
    let mut failures = Vec::new();
    // Newer versions may have syntax the parser doesn't know yet.
    for version in pgls_postgres_regress::versions()
        .into_iter()
        .filter(|x| x.major <= parser_major)
    {
        for file in version.files() {
            for statement in &file.statements {
                if statement.verdict != Verdict::Accepted {
                    continue;
                }
                checked += 1;
                if let Err(e) = pgls_query::parse(file.sql(statement)) {
                    let (line, col) = file.line_col(statement);
                    failures.push(format!(
                        "{}/{}.sql:{line}:{col}: {e}\n  {}",
                        version.major,
                        file.name,
                        file.sql(statement)
                    ));
                }
            }
        }
    }
    assert!(checked > 0, "no accepted statements to check");
    assert!(
        failures.is_empty(),
        "{} of {checked} accepted statements don't parse:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
