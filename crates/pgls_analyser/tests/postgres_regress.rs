//! Runs the type-check rules, and the lint rules that only report what Postgres rejects, over
//! Postgres' own regression suite of every supported version (`pgls_postgres_regress`), against
//! the catalog of a fresh database on that version.
//!
//! A finding on a statement Postgres accepted is a false positive and fails the test. Findings on
//! statements Postgres rejected are snapshotted per version, so a change in what we detect shows
//! up in review.
//!
//! The body of a `LANGUAGE sql` function is analysed like the workspace does it: right after the
//! `CREATE FUNCTION`, with the function's parameters in scope. A finding in the body of a function
//! Postgres accepted is a false positive too.
//!
//! The output columns that the resolver infers for the statements that return rows are compared
//! with the types Postgres described for them, see [`output_types_agree_with_postgres`], and the
//! catalog after each file with the objects of the database, see [`catalog_agrees_with_postgres`].

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use std::{collections::BTreeMap, sync::Arc};

use pgls_analyse::{AnalysisFilter, RuleFilter};
use pgls_analyser::{
    AnalysableStatement, AnalysedFileContext, Analyser, AnalyserConfig, AnalyserParams,
    LinterOptions, ResolveOptions, TYPECHECK_GROUP, UnparsableStatement, Walk, WalkParams,
    walk_statements,
};
use pgls_catalog::{
    Catalog, CatalogBase, CatalogCode, Decision, FunctionKind, Lookup, RelationKind, Snapshot,
    Type, TypeKind,
};
use pgls_console::StdDisplay;
use pgls_diagnostics::{Diagnostic, PrintDiagnostic};
use pgls_postgres_regress::{DatabaseObject, File, Verdict, Version};
use pgls_query::NodeEnum;
use pgls_resolve::FunctionContext;
use pgls_text_size::{TextRange, TextSize};
use rayon::prelude::*;

/// Findings on statements Postgres accepted only because an earlier statement of the file failed.
/// The linter assumes every statement succeeds, like a migration that stops at the first error.
const KNOWN_ARTEFACTS: &[(&str, &str, &str)] = &[
    (
        "transactions",
        "writetest",
        "`DROP TABLE writetest` fails in a read-only session, so the table still exists",
    ),
    (
        "foreign_key",
        "insert into fktable2 values (4, 5)",
        "`CREATE TEMP TABLE fktable2 (fk int references pktable ENFORCED NOT ENFORCED)` fails, so \
         no temporary `fktable2` shadows the one created later",
    ),
    (
        "constraints",
        "UPDATE notnull_tbl1 SET a = 100 WHERE b = 1",
        "`ALTER COLUMN a ADD GENERATED ALWAYS AS IDENTITY` fails because `a` has an invalid NOT \
         NULL constraint, so `a` is no identity column",
    ),
];

/// Known false positives in the bodies of SQL functions Postgres accepted: the file, a snippet of
/// the function, the rule, and the root cause. Remove an entry once it is fixed; an entry that
/// matches nothing fails the test.
const KNOWN_FALSE_POSITIVES: &[(&str, &str, &str, &str)] = &[];

#[test]
fn typecheck_reports_nothing_postgres_accepts() {
    let mut false_positives = Vec::new();
    let mut fixed = vec![true; KNOWN_FALSE_POSITIVES.len()];
    for version in pgls_postgres_regress::versions() {
        let catalog = catalog(&version);
        let files = version.files();
        let checked = files
            .par_iter()
            .map(|file| check(&catalog, file))
            .collect::<Vec<_>>();
        let analysed = Analysed {
            statements: files.iter().map(|x| x.statements.len()).sum(),
            body_statements: checked.iter().map(|x| x.body_statements).sum(),
        };
        let mut rejected = Vec::new();
        let mut artefacts = Vec::new();
        let mut known_false_positives = Vec::new();
        for finding in checked.into_iter().flat_map(|x| x.findings) {
            match &finding.verdict {
                Verdict::Accepted => {
                    if let Some(reason) = known_artefact(&finding) {
                        artefacts.push(format!("{}\n  {reason}", finding.location()));
                    } else if let Some(index) = known_false_positive(&finding) {
                        fixed[index] = false;
                        let (_, _, _, reason) = KNOWN_FALSE_POSITIVES[index];
                        known_false_positives.push(format!(
                            "{} {}\n  {reason}",
                            finding.location(),
                            finding.rule
                        ));
                    } else {
                        false_positives.push(format!(
                            "{}/{}: {}\n  {}\n  {}",
                            version.major,
                            finding.location(),
                            finding.rule,
                            finding.message,
                            truncate(&finding.sql, 300)
                        ));
                    }
                }
                Verdict::Rejected(state) => rejected.push((state.clone(), finding)),
                Verdict::Skipped => {}
            }
        }
        insta::assert_snapshot!(
            format!("typecheck_{}", version.major),
            report(
                &version,
                &analysed,
                &rejected,
                &artefacts,
                &known_false_positives
            )
        );
    }
    assert!(
        false_positives.is_empty(),
        "{} findings on statements Postgres accepted:\n\n{}",
        false_positives.len(),
        false_positives.join("\n\n")
    );
    let fixed = KNOWN_FALSE_POSITIVES
        .iter()
        .zip(fixed)
        .filter(|(_, fixed)| *fixed)
        .map(|((file, needle, rule, _), _)| format!("{file} {rule} `{needle}`"))
        .collect::<Vec<_>>();
    assert!(
        fixed.is_empty(),
        "known false positives that no longer occur, remove them from KNOWN_FALSE_POSITIVES:\n{}",
        fixed.join("\n")
    );
}

struct Analysed {
    statements: usize,
    /// The statements in SQL function bodies.
    body_statements: usize,
}

struct Checked {
    findings: Vec<Finding>,
    body_statements: usize,
}

struct Finding {
    file: String,
    /// Where the statement starts.
    line: usize,
    col: usize,
    /// Whether the finding is in the body of a SQL function.
    in_body: bool,
    rule: String,
    message: String,
    verdict: Verdict,
    sql: String,
}

impl Finding {
    fn location(&self) -> String {
        let body = if self.in_body { " (body)" } else { "" };
        format!("{}.sql:{}:{}{body}", self.file, self.line, self.col)
    }
}

fn catalog(version: &Version) -> Arc<CatalogBase> {
    let snapshot: Snapshot = serde_json::from_str(&version.catalog_json())
        .unwrap_or_else(|e| panic!("parse the catalog of {}: {e}", version.major));
    Arc::new(CatalogBase::new(Arc::new(snapshot)))
}

/// Lints a file like the workspace does: statements our parser doesn't understand (newer syntax)
/// are passed on as unparsable.
fn check(catalog: &Arc<CatalogBase>, file: &File) -> Checked {
    let options = LinterOptions::default();
    let rules = [
        RuleFilter::Group(TYPECHECK_GROUP),
        RuleFilter::Rule("correctness", "banDuplicateColumnAssignments"),
    ];
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter: AnalysisFilter::from_enabled_rules(&rules),
    });
    let Statements {
        stmts,
        unparsable,
        bodies,
    } = Statements::of(file, &file.statements);
    let diagnostics = analyser
        .analyse(AnalyserParams {
            stmts,
            unparsable,
            catalog_base: Some(catalog.clone()),
            search_path: vec!["public".into()],
            typecheck: true,
            ..Default::default()
        })
        .diagnostics;
    let findings = diagnostics
        .iter()
        .map(|diagnostic| {
            let span = diagnostic.location().span.expect("findings have a span");
            let statement = file
                .statements
                .iter()
                .find(|x| x.range.contains_inclusive(span.start()))
                .expect("findings are inside a statement");
            let (line, col) = file.line_col(statement);
            Finding {
                file: file.name.clone(),
                line,
                col,
                in_body: bodies
                    .iter()
                    .any(|body| body.contains_inclusive(span.start())),
                rule: diagnostic
                    .get_category_name()
                    .trim_start_matches("lint/")
                    .to_owned(),
                message: format!("{}", StdDisplay(PrintDiagnostic::simple(diagnostic)))
                    .replace('\n', " "),
                verdict: statement.verdict.clone(),
                sql: file.sql(statement).to_owned(),
            }
        })
        .collect();
    Checked {
        findings,
        body_statements: bodies.len(),
    }
}

/// The statements of a file as the workspace analyses them: statements our parser doesn't
/// understand (newer syntax) are passed on as unparsable, and the statements of the body of a SQL
/// function follow its `CREATE FUNCTION`.
struct Statements {
    stmts: Vec<AnalysableStatement>,
    unparsable: Vec<UnparsableStatement>,
    /// The ranges of the statements in SQL function bodies.
    bodies: Vec<TextRange>,
}

impl Statements {
    /// The statements of `file` among `statements`, in file order.
    fn of<'a>(
        file: &File,
        statements: impl IntoIterator<Item = &'a pgls_postgres_regress::Statement>,
    ) -> Self {
        let mut stmts = Vec::new();
        let mut unparsable = Vec::new();
        let mut bodies = Vec::new();
        for statement in statements {
            let sql = file.sql(statement);
            match parse(sql) {
                Ok(Some((root, has_identifier_parameters))) => {
                    let body = sql_function_body(&root, sql, statement.range);
                    stmts.push(
                        AnalysableStatement::new(root, statement.range)
                            .with_sql(sql)
                            .with_identifier_parameters(has_identifier_parameters),
                    );
                    bodies.extend(body.iter().map(|x| x.range));
                    stmts.extend(body);
                }
                Ok(None) => {}
                Err(()) => unparsable.push(UnparsableStatement {
                    range: statement.range,
                    sql: sql.to_owned(),
                }),
            }
        }
        Statements {
            stmts,
            unparsable,
            bodies,
        }
    }

    /// The statements of `file` that Postgres accepted, which are those that changed the
    /// database. The statements that Postgres skipped didn't run either, like transaction
    /// control, so the others ran outside of transaction blocks.
    fn accepted(file: &File) -> Self {
        Self::of(
            file,
            file.statements
                .iter()
                .filter(|statement| statement.verdict == Verdict::Accepted),
        )
    }

    /// Walks the statements like lint does, from the catalog of a fresh database.
    fn walk(
        &self,
        catalog: &Arc<CatalogBase>,
        visit: impl FnMut(&AnalysableStatement, &AnalysedFileContext),
    ) -> Walk {
        let statement_ranges = self.top_level_ranges();
        let search_path = ["public".to_owned()];
        walk_statements(
            WalkParams {
                stmts: &self.stmts,
                unparsable: &self.unparsable,
                statement_ranges: &statement_ranges,
                catalog_base: Some(catalog.clone()),
                search_path: &search_path,
                sections: &[],
                end: TextSize::from(u32::MAX),
                best_effort: false,
                from: None,
                checkpoints: &[],
                resolve: ResolveOptions::default(),
                should_stop: &|| false,
            },
            visit,
        )
    }

    /// The ranges of the top-level statements, parsed or not, in order.
    fn top_level_ranges(&self) -> Vec<TextRange> {
        let mut ranges = self
            .stmts
            .iter()
            .filter(|stmt| stmt.function.is_none())
            .map(|stmt| stmt.range)
            .chain(self.unparsable.iter().map(|x| x.range))
            .collect::<Vec<_>>();
        ranges.sort_unstable_by_key(|range| range.start());
        ranges
    }
}

/// The root of a statement, and whether it has psql identifier parameters. `Err` if it doesn't
/// parse, `None` if it isn't a single statement.
fn parse(sql: &str) -> Result<Option<(NodeEnum, bool)>, ()> {
    // psql variables (`:name`, `:'name'`) become positional parameters, as in the workspace.
    let conversion = pgls_lexer::convert_to_positional_params_with_metadata(sql);
    let ast = pgls_query::parse(&conversion.sql).map_err(|_| ())?;
    Ok(ast
        .into_root()
        .map(|root| (root, conversion.has_identifier_parameters)))
}

/// The statements in the body of a SQL function, as the workspace extracts and analyses them.
fn sql_function_body(root: &NodeEnum, sql: &str, range: TextRange) -> Vec<AnalysableStatement> {
    let NodeEnum::CreateFunctionStmt(create_fn) = root else {
        return Vec::new();
    };
    let function = FunctionContext::from_create_function(create_fn);
    pgls_query_ext::function_body::sql_function_body(root, sql)
        .iter()
        .filter_map(|statement| {
            let (root, has_identifier_parameters) = parse(&statement.sql).ok()??;
            Some(
                AnalysableStatement::embedded(root, range.start(), statement)
                    .with_function(function.clone())
                    .with_identifier_parameters(has_identifier_parameters),
            )
        })
        .collect()
}

fn known_false_positive(finding: &Finding) -> Option<usize> {
    KNOWN_FALSE_POSITIVES
        .iter()
        .position(|(file, needle, rule, _)| {
            finding.in_body
                && finding.file == *file
                && finding.rule == *rule
                && finding.sql.contains(needle)
        })
}

fn known_artefact(finding: &Finding) -> Option<&'static str> {
    KNOWN_ARTEFACTS
        .iter()
        .find(|(file, needle, _)| finding.file == *file && finding.sql.contains(needle))
        .map(|(_, _, reason)| *reason)
}

/// A summary per rule and SQLSTATE, the known artefacts and false positives, then one line per
/// finding on a rejected statement.
fn report(
    version: &Version,
    analysed: &Analysed,
    rejected: &[(String, Finding)],
    artefacts: &[String],
    false_positives: &[String],
) -> String {
    let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (state, finding) in rejected {
        *counts.entry((&finding.rule, state)).or_default() += 1;
    }
    let mut report = format!(
        "{}: {} statements and {} in SQL function bodies, {} findings on statements Postgres rejected\n\n",
        version.tag,
        analysed.statements,
        analysed.body_statements,
        rejected.len()
    );
    for ((rule, state), count) in counts {
        report.push_str(&format!("{rule} {state}: {count}\n"));
    }
    if !artefacts.is_empty() {
        report.push_str("\nKnown artefacts\n");
        for artefact in artefacts {
            report.push_str(&format!("{artefact}\n"));
        }
    }
    if !false_positives.is_empty() {
        report.push_str("\nKnown false positives\n");
        for false_positive in false_positives {
            report.push_str(&format!("{false_positive}\n"));
        }
    }
    report.push_str("\nFindings\n");
    let mut findings = rejected.iter().collect::<Vec<_>>();
    findings.sort_by(|(_, a), (_, b)| {
        (&a.file, a.line, a.col, &a.rule).cmp(&(&b.file, b.line, b.col, &b.rule))
    });
    for (state, finding) in findings {
        report.push_str(&format!(
            "{} {state} {}\n",
            finding.location(),
            finding.rule
        ));
    }
    report
}

fn truncate(sql: &str, max: usize) -> String {
    if sql.chars().count() <= max {
        sql.into()
    } else {
        format!("{}…", sql.chars().take(max).collect::<String>())
    }
}

/// Compares the output columns that the resolver infers with the types that Postgres described
/// for each accepted statement that returns rows ([`pgls_postgres_regress::Statement::output_types`]),
/// on every version, against the catalog of a fresh database on that version.
///
/// Each file is walked like lint walks it, over the statements Postgres accepted, so that each
/// statement sees what the statements that ran before it created. A column whose type the
/// resolver doesn't know is fine. A known type that
/// differs, or a different number of columns, is a mismatch. The mismatches are snapshotted per
/// version, so a new one fails the test and a fixed one shows up as a diff.
#[test]
fn output_types_agree_with_postgres() {
    for version in pgls_postgres_regress::versions() {
        let catalog = catalog(&version);
        let files = version.files();
        let compared = files
            .par_iter()
            .map(|file| OutputTypes::compare(&catalog, file))
            .collect::<Vec<_>>();
        let mut total = OutputTypes::default();
        for file in compared {
            total.add(file);
        }
        insta::assert_snapshot!(
            format!("output_types_{}", version.major),
            total.report(&version)
        );
    }
}

#[derive(Default)]
struct OutputTypes {
    /// The statements with recorded output types.
    statements: usize,
    /// The statements whose output columns the resolver doesn't know.
    unresolved: usize,
    /// The columns of the other statements whose type the resolver knows, and that have the
    /// type Postgres described.
    agree: usize,
    /// The columns of the other statements whose type the resolver doesn't know.
    unknown: usize,
    /// One entry per statement whose columns differ from what Postgres described.
    mismatches: Vec<String>,
}

impl OutputTypes {
    fn compare(catalog: &Arc<CatalogBase>, file: &File) -> Self {
        let mut compared = OutputTypes::default();
        Statements::accepted(file).walk(catalog, |stmt, file_context| {
            if stmt.function.is_some() || stmt.has_identifier_parameters {
                return;
            }
            let Some((statement, recorded)) = file
                .statements
                .binary_search_by_key(&stmt.range.start(), |x| x.range.start())
                .ok()
                .and_then(|index| file.statements.get(index))
                .and_then(|x| Some((x, x.output_types.as_ref()?)))
            else {
                return;
            };
            compared.statements += 1;
            let Some(columns) = file_context
                .resolution(stmt)
                .output
                .as_ref()
                .and_then(|output| output.columns())
            else {
                compared.unresolved += 1;
                return;
            };
            let inferred = columns
                .iter()
                .map(|column| {
                    let ty = column.ty.as_ref()?;
                    Self::type_name(file_context.catalog(), ty)
                })
                .collect::<Vec<_>>();
            let differs = inferred.len() != recorded.len()
                || inferred
                    .iter()
                    .zip(recorded)
                    .any(|(inferred, recorded)| inferred.as_ref().is_some_and(|x| x != recorded));
            if differs {
                let (line, col) = file.line_col(statement);
                let inferred = inferred
                    .iter()
                    .map(|x| x.as_deref().unwrap_or("?"))
                    .collect::<Vec<_>>();
                compared.mismatches.push(format!(
                    "{}.sql:{line}:{col}: inferred {}, Postgres {}\n  {}",
                    file.name,
                    inferred.join(", "),
                    recorded.join(", "),
                    truncate(&file.sql(statement).replace('\n', " "), 200)
                ));
            } else {
                compared.unknown += inferred.iter().filter(|x| x.is_none()).count();
                compared.agree += inferred.iter().filter(|x| x.is_some()).count();
            }
        });
        compared
    }

    /// The type that Postgres describes a column of type `ty` by, as `schema.name`, if it is
    /// known.
    fn type_name(catalog: &Catalog, ty: &Type) -> Option<String> {
        let id = match ty {
            // An unknown literal in a select list or RETURNING list becomes `text`
            // ([`resolveTargetListUnknowns`]).
            //
            // [`resolveTargetListUnknowns`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L288
            Type::UnknownLiteral => return Some("pg_catalog.text".to_owned()),
            Type::Record(_) => return Some("pg_catalog.record".to_owned()),
            Type::Named(id) => id,
        };
        // A column of a domain is described by the domain's base type
        // ([`SendRowDescriptionMessage`]).
        //
        // [`SendRowDescriptionMessage`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/common/printtup.c#L166
        let Decision::Known(base) = pgls_catalog::base_type(catalog, id) else {
            return None;
        };
        CatalogObjects::type_name(catalog, &Type::Named(base))
    }

    fn add(&mut self, other: OutputTypes) {
        self.statements += other.statements;
        self.unresolved += other.unresolved;
        self.agree += other.agree;
        self.unknown += other.unknown;
        self.mismatches.extend(other.mismatches);
    }

    fn report(&self, version: &Version) -> String {
        let mut report = format!(
            "{}: {} statements that return rows, {} without inferred output columns. Of the \
             columns of the others, {} have the type Postgres described, {} an unknown type. {} \
             statements differ\n",
            version.tag,
            self.statements,
            self.unresolved,
            self.agree,
            self.unknown,
            self.mismatches.len()
        );
        for mismatch in &self.mismatches {
            report.push('\n');
            report.push_str(mismatch);
            report.push('\n');
        }
        report
    }
}

/// Compares the catalog after each file with the objects of the database after Postgres ran it
/// ([`pgls_postgres_regress::File::objects`]), on every version, from the catalog of a fresh
/// database on that version.
///
/// The file is walked over the statements Postgres accepted, like the database ran them. Each
/// object of the database is looked up in the catalog:
/// - One that the catalog knows to be missing, or whose kind, columns, column types or number of
///   overloads differ, is a mismatch.
/// - One the catalog can't tell about is fine, and so is a column whose type it doesn't know, or
///   a relation whose columns it doesn't know.
///
/// A schema, relation, type or function name that the catalog has and the database doesn't is a
/// mismatch too. Temporary objects are left out: the session that created them ended. The
/// mismatches are snapshotted per version, so a new one fails the test and a fixed one shows up
/// as a diff.
#[test]
fn catalog_agrees_with_postgres() {
    for version in pgls_postgres_regress::versions() {
        let catalog = catalog(&version);
        let files = version.files();
        let compared = files
            .par_iter()
            .map(|file| CatalogObjects::compare(&catalog, file))
            .collect::<Vec<_>>();
        let mut total = CatalogObjects::default();
        for file in compared {
            total.add(file);
        }
        insta::assert_snapshot!(format!("catalog_{}", version.major), total.report(&version));
    }
}

#[derive(Default)]
struct CatalogObjects {
    /// The objects of the databases after the files.
    objects: usize,
    /// The objects that the catalog has like the database.
    agree: usize,
    /// The objects that the catalog can't tell about, or only in part.
    unknown: usize,
    /// One entry per object that differs, is missing or is extra.
    mismatches: Vec<String>,
}

/// How an object of the database compares with the catalog.
enum Comparison {
    Agrees,
    Unknown,
    Differs(String),
}

impl CatalogObjects {
    fn compare(catalog: &Arc<CatalogBase>, file: &File) -> Self {
        let walk = Statements::accepted(file).walk(catalog, |_, _| {});
        let catalog = &walk.catalog;
        let mut compared = CatalogObjects::default();
        for object in &file.objects {
            compared.objects += 1;
            match Self::compare_object(catalog, object) {
                Comparison::Agrees => compared.agree += 1,
                Comparison::Unknown => compared.unknown += 1,
                Comparison::Differs(difference) => compared
                    .mismatches
                    .push(format!("{}.sql: {difference}", file.name)),
            }
        }
        for extra in Self::extra(catalog, &file.objects) {
            compared
                .mismatches
                .push(format!("{}.sql: {extra} isn't in the database", file.name));
        }
        compared
    }

    fn compare_object(catalog: &Catalog, object: &DatabaseObject) -> Comparison {
        match object {
            DatabaseObject::Schema { name } => match catalog.schema(name) {
                Lookup::Found(()) => Comparison::Agrees,
                Lookup::Unknown => Comparison::Unknown,
                Lookup::Missing => Comparison::Differs(format!("schema {name} is missing")),
            },
            DatabaseObject::Relation {
                kind,
                schema,
                name,
                columns,
            } => {
                let relation = match catalog.relation(Some(schema), name, &[]) {
                    Lookup::Found(relation) => relation,
                    Lookup::Unknown => return Comparison::Unknown,
                    Lookup::Missing => {
                        return Comparison::Differs(format!("relation {schema}.{name} is missing"));
                    }
                };
                let expected = match kind {
                    'r' => RelationKind::Table,
                    'p' => RelationKind::PartitionedTable,
                    'v' => RelationKind::View,
                    'm' => RelationKind::MaterializedView,
                    'f' => RelationKind::ForeignTable,
                    _ => RelationKind::Other,
                };
                if relation.kind != expected {
                    return Comparison::Differs(format!(
                        "relation {schema}.{name} is a {:?}, not a {expected:?}",
                        relation.kind
                    ));
                }
                let Some(inferred) = &relation.columns else {
                    return Comparison::Unknown;
                };
                let inferred = inferred
                    .iter()
                    .map(|column| {
                        let ty = column
                            .ty
                            .as_ref()
                            .and_then(|ty| Self::type_name(catalog, ty));
                        (column.name.as_str(), ty)
                    })
                    .collect::<Vec<_>>();
                let differs = inferred.len() != columns.len()
                    || inferred.iter().zip(columns).any(|((name, ty), column)| {
                        *name != column.0 || ty.as_ref().is_some_and(|ty| *ty != column.1)
                    });
                if differs {
                    let inferred = inferred
                        .iter()
                        .map(|(name, ty)| format!("{name} {}", ty.as_deref().unwrap_or("?")))
                        .collect::<Vec<_>>();
                    let columns = columns
                        .iter()
                        .map(|(name, ty)| format!("{name} {ty}"))
                        .collect::<Vec<_>>();
                    Comparison::Differs(format!(
                        "relation {schema}.{name} has the columns ({}), not ({})",
                        inferred.join(", "),
                        columns.join(", ")
                    ))
                } else if inferred.iter().any(|(_, ty)| ty.is_none()) {
                    Comparison::Unknown
                } else {
                    Comparison::Agrees
                }
            }
            DatabaseObject::Type { kind, schema, name } => {
                match catalog.type_(Some(schema), name, &[]) {
                    Lookup::Found(info) => match info.kind {
                        None => Comparison::Unknown,
                        Some(inferred) if inferred.code() == kind.to_string() => Comparison::Agrees,
                        Some(inferred) => Comparison::Differs(format!(
                            "type {schema}.{name} is a {inferred:?}, not of kind {kind}"
                        )),
                    },
                    Lookup::Unknown => Comparison::Unknown,
                    Lookup::Missing => {
                        Comparison::Differs(format!("type {schema}.{name} is missing"))
                    }
                }
            }
            DatabaseObject::Function {
                kind,
                schema,
                name,
                overloads,
            } => {
                let functions = match catalog.functions(Some(schema), name, &[]) {
                    Lookup::Found(functions) => functions,
                    Lookup::Unknown => return Comparison::Unknown,
                    Lookup::Missing => Vec::new(),
                };
                let expected = match kind {
                    'p' => FunctionKind::Procedure,
                    'a' => FunctionKind::Aggregate,
                    'w' => FunctionKind::Window,
                    _ => FunctionKind::Function,
                };
                let inferred = functions
                    .iter()
                    .filter(|function| function.kind == expected)
                    .count();
                if inferred == *overloads {
                    Comparison::Agrees
                } else {
                    Comparison::Differs(format!(
                        "{expected:?} {schema}.{name} has {inferred} overloads, not {overloads}"
                    ))
                }
            }
        }
    }

    /// The schemas, relations, types and functions that the catalog has and that aren't objects
    /// of the database, outside of the system schemas and `pg_temp`. The types are those that
    /// the database's objects list: no row types of relations, arrays, multiranges or shell
    /// types. Only objects that the catalog's lookups find are listed, not those it can't tell
    /// about.
    fn extra(catalog: &Catalog, objects: &[DatabaseObject]) -> Vec<String> {
        let Some(snapshot) = catalog.snapshot() else {
            return Vec::new();
        };
        let listed = |schema: &str| {
            !matches!(
                schema,
                "pg_catalog" | "information_schema" | "pg_toast" | "pg_temp"
            )
        };
        let has = |category: &str, schema: &str, name: &str| {
            objects.iter().any(|object| match object {
                DatabaseObject::Schema { name: x } => category == "schema" && x == name,
                DatabaseObject::Relation {
                    schema: s, name: n, ..
                } => category == "relation" && s == schema && n == name,
                DatabaseObject::Type {
                    schema: s, name: n, ..
                } => category == "type" && s == schema && n == name,
                DatabaseObject::Function {
                    schema: s, name: n, ..
                } => category == "function" && s == schema && n == name,
            })
        };
        let mut extra = Vec::new();
        for schema in &snapshot.schemas {
            if listed(&schema.name)
                && !has("schema", "", &schema.name)
                && matches!(catalog.schema(&schema.name), Lookup::Found(()))
            {
                extra.push(format!("schema {}", schema.name));
            }
        }
        for table in &snapshot.tables {
            if listed(&table.schema)
                && !has("relation", &table.schema, &table.name)
                && matches!(
                    catalog.relation(Some(&table.schema), &table.name, &[]),
                    Lookup::Found(_)
                )
            {
                extra.push(format!("relation {}.{}", table.schema, table.name));
            }
        }
        for type_ in &snapshot.types {
            let is_row_type = snapshot
                .tables
                .iter()
                .any(|table| table.schema == type_.schema && table.name == type_.name);
            if listed(&type_.schema)
                && !type_.is_array
                && !matches!(
                    type_.typtype.get(),
                    Some(TypeKind::Multirange | TypeKind::Pseudo)
                )
                && !is_row_type
                && !has("type", &type_.schema, &type_.name)
                && matches!(
                    catalog.type_(Some(&type_.schema), &type_.name, &[]),
                    Lookup::Found(_)
                )
            {
                extra.push(format!("type {}.{}", type_.schema, type_.name));
            }
        }
        let mut functions = snapshot
            .functions
            .iter()
            .filter(|function| listed(&function.schema))
            .map(|function| (function.schema.as_str(), function.name.as_str()))
            .collect::<Vec<_>>();
        functions.sort_unstable();
        functions.dedup();
        for (schema, name) in functions {
            if !has("function", schema, name)
                && matches!(catalog.functions(Some(schema), name, &[]), Lookup::Found(_))
            {
                extra.push(format!("function {schema}.{name}"));
            }
        }
        // The catalog's objects come in the order of its hash maps.
        extra.sort_unstable();
        extra
    }

    /// A type as `schema.name`, if it is known. An unknown literal is `pg_catalog.unknown`.
    fn type_name(catalog: &Catalog, ty: &Type) -> Option<String> {
        match ty {
            Type::UnknownLiteral => Some("pg_catalog.unknown".to_owned()),
            Type::Record(_) => Some("pg_catalog.record".to_owned()),
            Type::Named(id) => {
                let info = catalog.type_by_id(id).found()?;
                Some(format!("{}.{}", info.schema, info.name))
            }
        }
    }

    fn add(&mut self, other: CatalogObjects) {
        self.objects += other.objects;
        self.agree += other.agree;
        self.unknown += other.unknown;
        self.mismatches.extend(other.mismatches);
    }

    fn report(&self, version: &Version) -> String {
        let mut report = format!(
            "{}: {} objects of the databases after the files, {} as in the catalog, {} the \
             catalog can't tell about. {} mismatches\n\n",
            version.tag,
            self.objects,
            self.agree,
            self.unknown,
            self.mismatches.len()
        );
        for mismatch in &self.mismatches {
            report.push_str(mismatch);
            report.push('\n');
        }
        report
    }
}
