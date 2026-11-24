mod diagnostics;

use std::borrow::Cow;

pub use diagnostics::PlPgSqlCheckDiagnostic;
use pgls_catalog::{NamePosition, quote_identifier};
use pgls_query::protobuf::{CreateFunctionStmt, FunctionParameterMode, Token, TypeName};
use pgls_query_ext::{
    function_body::{Language, OffsetMap, function_body_location},
    utils::{QualifiedName, find_option_value, qualified_name},
};
use pgls_text_size::TextRange;
use serde::Deserialize;
use sqlx::{Acquire, PgPool, Postgres, Transaction};

#[derive(Debug)]
pub struct PlPgSqlCheckParams<'a> {
    pub conn: &'a PgPool,
    pub sql: &'a str,
    pub ast: &'a pgls_query::NodeEnum,
    pub snapshot: &'a pgls_catalog::Snapshot,
    pub fatal_errors: bool,
    pub other_warnings: bool,
    pub extra_warnings: bool,
    pub performance_warnings: bool,
    pub security_warnings: bool,
    pub compatibility_warnings: bool,
    pub without_warnings: bool,
    pub all_warnings: bool,
    pub use_incomment_options: bool,
    pub incomment_options_usage_warning: bool,
    pub constant_tracing: bool,
}

/// What [`check`] did with a statement.
#[derive(Debug)]
pub enum Checked {
    /// plpgsql_check checked the function and found these issues.
    Issues(Vec<PlPgSqlCheckDiagnostic>),
    /// The statement wasn't checked.
    Skipped(SkipReason),
}

/// Why [`check`] didn't check a statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// It doesn't create a PL/pgSQL function.
    NotPlPgSql,
    /// The extension isn't installed in the database.
    ExtensionMissing,
    /// A trigger function that no trigger of the snapshot uses. plpgsql_check checks a trigger
    /// function for the table of a trigger.
    TriggerWithoutTable,
    /// An argument's type is written as `%TYPE`, which a signature can't name.
    ArgumentTypeReference,
    /// The body isn't a string constant found in the statement, like a string with an invalid
    /// escape, which Postgres rejects anyway.
    BodyNotFound,
}

/// Why [`check`] failed.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("plpgsql_check returned output that isn't its JSON: {0}")]
    Output(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
pub(crate) struct PlPgSqlCheckResult {
    pub(crate) issues: Vec<PlPgSqlCheckIssue>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PlPgSqlCheckIssue {
    pub(crate) level: String,
    pub(crate) message: String,
    pub(crate) detail: Option<String>,
    pub(crate) hint: Option<String>,
    pub(crate) statement: Option<Statement>,
    pub(crate) query: Option<Query>,
    #[serde(rename = "sqlState")]
    pub(crate) sql_state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Statement {
    #[serde(rename = "lineNumber")]
    pub(crate) line_number: String,
    pub(crate) text: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Query {
    pub(crate) position: String,
    pub(crate) text: String,
}

/// The body of a function: its value, which plpgsql_check's positions are in, and where it is
/// in the statement.
pub(crate) struct FunctionBody<'a> {
    pub(crate) value: &'a str,
    /// Where the string constant's text is in the statement.
    range: TextRange,
    /// How offsets in `value` map to the constant's text, which differ where it escapes
    /// characters.
    offsets: OffsetMap,
}

impl FunctionBody<'_> {
    /// The range in the statement of `range` in the body's value.
    pub(crate) fn to_statement(&self, range: TextRange) -> TextRange {
        self.offsets.range_to_source(range) + self.range.start()
    }
}

/// A PL/pgSQL function that a statement creates, which plpgsql_check can check.
struct PlPgSqlFunction<'a> {
    create_fn: &'a CreateFunctionStmt,
    schema: Option<&'a str>,
    name: &'a str,
    body: FunctionBody<'a>,
    /// The schema of the plpgsql_check extension, whose functions check this one.
    extension_schema: Option<&'a str>,
}

impl<'a> PlPgSqlFunction<'a> {
    /// The PL/pgSQL function that `ast`, whose text is `sql`, creates, if plpgsql_check can check
    /// it in the database of `snapshot`.
    fn of(
        ast: &'a pgls_query::NodeEnum,
        sql: &str,
        snapshot: &'a pgls_catalog::Snapshot,
    ) -> Result<Self, SkipReason> {
        let pgls_query::NodeEnum::CreateFunctionStmt(create_fn) = ast else {
            return Err(SkipReason::NotPlPgSql);
        };
        if Language::of(create_fn) != Some(Language::PlPgSql) {
            return Err(SkipReason::NotPlPgSql);
        }
        // Only an installed extension has `plpgsql_check_function`.
        let extension = snapshot
            .extension("plpgsql_check")
            .filter(|extension| extension.is_installed())
            .ok_or(SkipReason::ExtensionMissing)?;
        let Some(QualifiedName { schema, name, .. }) = qualified_name(&create_fn.funcname) else {
            return Err(SkipReason::NotPlPgSql);
        };
        // The body's value is the `AS` option; the scanner finds its constant, so that an
        // escaped body or one whose text also occurs earlier in the statement is found.
        let value = find_option_value(create_fn, "as").ok_or(SkipReason::BodyNotFound)?;
        let (range, offsets) =
            function_body_location(create_fn, sql).ok_or(SkipReason::BodyNotFound)?;
        debug_assert!(
            usize::from(range.end()) <= sql.len(),
            "the body is in the statement"
        );
        Ok(Self {
            create_fn,
            schema,
            name,
            body: FunctionBody {
                value,
                range,
                offsets,
            },
            extension_schema: extension.schema.as_deref(),
        })
    }

    /// `plpgsql_check_function`, qualified with the schema of the extension, which needn't be
    /// on the search path, like `extensions` on Supabase.
    fn check_function(&self) -> String {
        match self.extension_schema {
            Some(schema) => format!(
                "{}.plpgsql_check_function",
                quote_identifier(schema, NamePosition::ColId)
            ),
            None => "plpgsql_check_function".to_string(),
        }
    }

    fn is_trigger(&self) -> bool {
        self.create_fn.return_type.as_ref().is_some_and(|n| {
            matches!(
                qualified_name(&n.names),
                Some(QualifiedName {
                    schema: None,
                    name: "trigger",
                    ..
                })
            )
        })
    }

    /// The signature of the function as `regprocedure` reads it ([`regprocedurein`]): the
    /// quoted name and the types of the arguments that identify it. A function's `OUT` and
    /// `TABLE` arguments don't, a procedure's `OUT` arguments do ([`interpret_function_parameter_list`]).
    /// `None` if an argument's type is a `%TYPE` reference.
    ///
    /// [`regprocedurein`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/adt/regproc.c#L227
    /// [`interpret_function_parameter_list`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/functioncmds.c#L184
    fn signature(&self) -> Option<String> {
        let mut signature = match self.schema {
            Some(schema) => format!(
                "{}.{}",
                quote_identifier(schema, NamePosition::ColId),
                quote_identifier(self.name, NamePosition::ColLabel)
            ),
            None => quote_identifier(self.name, NamePosition::TypeFunctionName).into_owned(),
        };
        signature.push('(');
        let mut first = true;
        for parameter in &self.create_fn.parameters {
            let Some(pgls_query::NodeEnum::FunctionParameter(parameter)) = &parameter.node else {
                continue;
            };
            let identifies = match parameter.mode() {
                FunctionParameterMode::FuncParamOut => self.create_fn.is_procedure,
                FunctionParameterMode::FuncParamTable => false,
                _ => true,
            };
            if !identifies {
                continue;
            }
            let type_name = parameter.arg_type.as_ref()?;
            if !first {
                signature.push_str(", ");
            }
            first = false;
            signature.push_str(&Self::type_name(type_name)?);
        }
        signature.push(')');
        Some(signature)
    }

    /// The type `type_name` as a type name in a signature: its quoted name, `[]` for an array.
    /// Postgres ignores the type modifiers of an argument.
    fn type_name(type_name: &TypeName) -> Option<String> {
        if type_name.pct_type {
            return None;
        }
        let mut name = String::new();
        for (i, part) in type_name.names.iter().enumerate() {
            let Some(pgls_query::NodeEnum::String(part)) = &part.node else {
                return None;
            };
            let position = if i == 0 {
                NamePosition::TypeFunctionName
            } else {
                name.push('.');
                NamePosition::ColLabel
            };
            name.push_str(&quote_identifier(&part.sval, position));
        }
        if !type_name.array_bounds.is_empty() {
            name.push_str("[]");
        }
        Some(name)
    }

    /// `sql` as `CREATE OR REPLACE`, so that it replaces the function if the database has it:
    /// `OR REPLACE` inserted before the `FUNCTION` or `PROCEDURE` keyword.
    fn create_or_replace<'s>(&self, sql: &'s str) -> Cow<'s, str> {
        if self.create_fn.replace {
            return Cow::Borrowed(sql);
        }
        let keyword = pgls_query::scan(sql).ok().and_then(|scan| {
            scan.tokens
                .iter()
                .find(|token| matches!(token.token(), Token::Function | Token::Procedure))
                .and_then(|token| usize::try_from(token.start).ok())
        });
        match keyword.and_then(|start| Some((sql.get(..start)?, sql.get(start..)?))) {
            Some((before, after)) => Cow::Owned(format!("{before}OR REPLACE {after}")),
            None => Cow::Borrowed(sql),
        }
    }
}

impl PlPgSqlCheckParams<'_> {
    /// The named arguments of `plpgsql_check_function` for the options that differ from its
    /// defaults, so that older versions of the extension, which lack some of them, still work.
    fn extra_arguments(&self) -> String {
        let arguments = [
            (!self.fatal_errors, "fatal_errors := false"),
            (!self.other_warnings, "other_warnings := false"),
            (!self.extra_warnings, "extra_warnings := false"),
            (self.performance_warnings, "performance_warnings := true"),
            (self.security_warnings, "security_warnings := true"),
            (
                self.compatibility_warnings,
                "compatibility_warnings := true",
            ),
            (self.without_warnings, "without_warnings := true"),
            (self.all_warnings, "all_warnings := true"),
            (
                !self.use_incomment_options,
                "use_incomment_options := false",
            ),
            (
                self.incomment_options_usage_warning,
                "incomment_options_usage_warning := true",
            ),
            (!self.constant_tracing, "constant_tracing := false"),
        ];
        arguments
            .into_iter()
            .filter(|(differs, _)| *differs)
            .map(|(_, argument)| format!(", {argument}"))
            .collect()
    }
}

/// Checks a PL/pgSQL function with plpgsql_check: creates it in a transaction that is rolled
/// back, and checks it, a trigger function once for the table of each trigger that uses it.
pub async fn check(params: PlPgSqlCheckParams<'_>) -> Result<Checked, CheckError> {
    let function = match PlPgSqlFunction::of(params.ast, params.sql, params.snapshot) {
        Ok(function) => function,
        Err(reason) => return Ok(Checked::Skipped(reason)),
    };
    let Some(signature) = function.signature() else {
        return Ok(Checked::Skipped(SkipReason::ArgumentTypeReference));
    };

    // The tables of the triggers that use a trigger function, as `regclass` reads them.
    let relations: Vec<String> = if function.is_trigger() {
        let relations: Vec<String> = params
            .snapshot
            .triggers
            .iter()
            .filter(|trigger| {
                trigger.proc_name == function.name
                    && function
                        .schema
                        .is_none_or(|schema| schema == trigger.proc_schema)
            })
            .map(|trigger| {
                format!(
                    "{}.{}",
                    quote_identifier(&trigger.table_schema, NamePosition::ColId),
                    quote_identifier(&trigger.table_name, NamePosition::ColLabel)
                )
            })
            .collect();
        if relations.is_empty() {
            return Ok(Checked::Skipped(SkipReason::TriggerWithoutTable));
        }
        relations
    } else {
        Vec::new()
    };

    // A connection of the pool. The function only exists in the transaction, which is rolled
    // back, and the queries that name it aren't prepared (`persistent(false)`), so the connection
    // keeps nothing of it for the next function.
    let mut conn = params.conn.acquire().await?;

    let mut tx: Transaction<'_, Postgres> = conn.begin().await?;

    // disable function body checking to rely on plpgsql_check
    sqlx::query("SET LOCAL check_function_bodies = off")
        .execute(&mut *tx)
        .await?;

    // create the function - this should always succeed
    sqlx::query(&function.create_or_replace(params.sql))
        .persistent(false)
        .execute(&mut *tx)
        .await?;

    // The names are bound as text and read by `regprocedure` and `regclass`, so that quoted
    // names work and no name ends up in the query.
    let check_function = function.check_function();
    let extra = params.extra_arguments();
    let mut results: Vec<(String, Option<String>)> = Vec::new();
    if relations.is_empty() {
        let result: Option<String> = sqlx::query_scalar(&format!(
            "select {check_function}($1::regprocedure, format := 'json'{extra})"
        ))
        .bind(&signature)
        .persistent(false)
        .fetch_optional(&mut *tx)
        .await?
        .flatten();
        results.extend(result.map(|result| (result, None)));
    } else {
        for relation in relations {
            let result: Option<String> = sqlx::query_scalar(&format!(
                "select {check_function}($1::regprocedure, $2::regclass, format := 'json'{extra})"
            ))
            .bind(&signature)
            .bind(&relation)
            .persistent(false)
            .fetch_optional(&mut *tx)
            .await?
            .flatten();
            results.extend(result.map(|result| (result, Some(relation))));
        }
    }

    tx.rollback().await?;

    let mut diagnostics = Vec::new();
    for (result, relation) in results {
        let result: PlPgSqlCheckResult = serde_json::from_str(&result)?;
        diagnostics.extend(result.diagnostics(&function.body, relation));
    }
    Ok(Checked::Issues(diagnostics))
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use sqlx::{Executor, PgPool};

    /// Test helper to run plpgsql_check and return diagnostics with span text
    async fn run_plpgsql_check_test(
        test_db: &PgPool,
        setup_sql: &str,
        create_fn_sql: &str,
    ) -> Result<(Vec<super::PlPgSqlCheckDiagnostic>, Vec<Option<String>>), Box<dyn std::error::Error>>
    {
        test_db.execute(setup_sql).await?;

        let ast = pgls_query::parse(create_fn_sql)?
            .into_root()
            .ok_or("Failed to parse SQL root")?;
        let snapshot = pgls_catalog::Snapshot::load(test_db).await?;

        let checked = super::check(super::PlPgSqlCheckParams {
            conn: test_db,
            sql: create_fn_sql,
            ast: &ast,
            snapshot: &snapshot,
            fatal_errors: true,
            other_warnings: true,
            extra_warnings: true,
            performance_warnings: false,
            security_warnings: false,
            compatibility_warnings: false,
            without_warnings: false,
            all_warnings: false,
            use_incomment_options: true,
            incomment_options_usage_warning: false,
            constant_tracing: true,
        })
        .await?;
        let diagnostics = match checked {
            super::Checked::Issues(diagnostics) => diagnostics,
            super::Checked::Skipped(reason) => return Err(format!("skipped: {reason:?}").into()),
        };

        let span_texts = diagnostics
            .iter()
            .map(|diag| {
                diag.span.as_ref().map(|s| {
                    let start = usize::from(s.start());
                    let end = usize::from(s.end());
                    create_fn_sql[start..end].to_string()
                })
            })
            .collect();

        Ok((diagnostics, span_texts))
    }

    /// The extension is available on the test server, but this database doesn't install it, so
    /// the function isn't checked. It used to be created and then fail on
    /// `plpgsql_check_function`.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_available_but_not_installed(test_db: PgPool) {
        let setup = "drop extension if exists plpgsql_check;";
        let create_fn_sql = r#"
            create function not_installed() returns int language plpgsql as $$
            begin
                return missing_var;
            end;
            $$;
        "#;
        test_db
            .execute(setup)
            .await
            .expect("Unable to set up the database");

        let ast = pgls_query::parse(create_fn_sql)
            .expect("Unable to parse")
            .into_root()
            .expect("Missing root");
        let snapshot = pgls_catalog::Snapshot::load(&test_db)
            .await
            .expect("Unable to load the snapshot");
        assert!(
            snapshot
                .extension("plpgsql_check")
                .is_some_and(|extension| !extension.is_installed()),
            "the test server should have plpgsql_check available"
        );

        let result = super::check(super::PlPgSqlCheckParams {
            conn: &test_db,
            sql: create_fn_sql,
            ast: &ast,
            snapshot: &snapshot,
            fatal_errors: true,
            other_warnings: true,
            extra_warnings: true,
            performance_warnings: false,
            security_warnings: false,
            compatibility_warnings: false,
            without_warnings: false,
            all_warnings: false,
            use_incomment_options: true,
            incomment_options_usage_warning: false,
            constant_tracing: true,
        })
        .await
        .expect("plpgsql_check must not run");

        assert!(
            matches!(
                result,
                super::Checked::Skipped(super::SkipReason::ExtensionMissing)
            ),
            "{result:?}"
        );
    }

    /// The body is found where the statement has it, also when it escapes quotes, and when its
    /// text occurs earlier in the statement. Both used to leave the function unchecked or put
    /// the diagnostic at the wrong place.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_finds_the_body_by_location(test_db: PgPool) {
        let setup = "create extension if not exists plpgsql_check;";

        let escaped = "create function escaped() returns int language plpgsql as 'begin raise notice ''it''''s''; return missing_var; end';";
        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, escaped)
            .await
            .expect("Failed to run plpgsql_check test");
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(span_texts[0].as_deref(), Some("missing_var"));

        let repeated = "create /* begin return missing_var; end; */ function repeated() returns int language plpgsql as $$begin return missing_var; end;$$;";
        let (diagnostics, _) = run_plpgsql_check_test(&test_db, setup, repeated)
            .await
            .expect("Failed to run plpgsql_check test");
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        let body = repeated.find("$$").expect("a body");
        let span = diagnostics[0].span.expect("a span");
        assert!(usize::from(span.start()) > body, "{span:?}");
        assert_eq!(
            &repeated[std::ops::Range::<usize>::from(span)],
            "missing_var"
        );
    }

    /// Names that need quotes, and a quote in a name, are passed to the database as they are.
    /// `OUT` arguments aren't part of a function's signature.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_quoted_names(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;
            create schema "My Schema";
            create type "My Schema"."My Type" as (a int);
        "#;
        let create_fn_sql = r#"
            create function "My Schema"."Check 'Me'"(p "My Schema"."My Type", p2 int[], out q int)
            language plpgsql as $$
            begin
                q := missing_var;
            end;
            $$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(span_texts[0].as_deref(), Some("missing_var"));
    }

    /// A trigger function is checked for a table whose name needs quotes.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_trigger_on_a_quoted_table(test_db: PgPool) {
        let create_fn_sql = r#"
            create or replace function public.audit_trigger() returns trigger
            language plpgsql as $$
            begin
                new.missing := 1;
                return new;
            end;
            $$;
        "#;
        let setup = format!(
            r#"
            create extension if not exists plpgsql_check;
            create table public."Audit Items" (id int);
            {create_fn_sql}
            create trigger audit before insert on public."Audit Items"
                for each row execute function public.audit_trigger();
            "#
        );

        let (diagnostics, _) = run_plpgsql_check_test(&test_db, &setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(
            diagnostics[0].advices.relation.as_deref(),
            Some(r#"public."Audit Items""#)
        );
    }

    /// A procedure that the database already has is replaced: `OR REPLACE` goes before the
    /// keyword, after a comment. A procedure's `OUT` arguments are part of its signature.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_existing_procedure(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;
            create procedure public.existing(a int, out b int) language plpgsql as $$
            begin
                b := a;
            end;
            $$;
        "#;
        let create_fn_sql = r#"
            create /* replaces it */ procedure public.existing(a int, out b int)
            language plpgsql as $$
            begin
                b := missing_var;
            end;
            $$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(span_texts[0].as_deref(), Some("missing_var"));
    }

    /// The extension's functions are called in its schema, which isn't on the search path.
    /// Every check used to fail there, and the error was dropped.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_extension_in_another_schema(test_db: PgPool) {
        let setup = "create schema \"Ext Schema\"; create extension if not exists plpgsql_check schema \"Ext Schema\";";
        let create_fn_sql = "create function in_other_schema() returns int language plpgsql as $$ begin return missing_var; end; $$;";

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(span_texts[0].as_deref(), Some("missing_var"));
    }

    /// A signature can't name a `%TYPE` argument, so the function isn't checked.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_skips_type_references(test_db: PgPool) {
        let setup =
            "create extension if not exists plpgsql_check; create table public.items (id int);";
        let create_fn_sql = "create function by_type(p public.items.id%type) returns int language plpgsql as $$ begin return missing_var; end; $$;";

        let result = run_plpgsql_check_test(&test_db, setup, create_fn_sql).await;

        assert_eq!(
            result
                .map(|(diagnostics, _)| diagnostics.len())
                .map_err(|err| err.to_string()),
            Err("skipped: ArgumentTypeReference".to_string())
        );
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_composite_types(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            create table if not exists _fetch_cycle_continuation_data (
                next_id bigint,
                next_state jsonb null default '{}'::jsonb
                constraint abstract_no_data check(false) no inherit
            );
        "#;

        let create_fn_sql = r#"
            create or replace function continue_fetch_cycle_prototype (
            ) returns _fetch_cycle_continuation_data language plpgsql as $prototype$
            declare
                result _fetch_cycle_continuation_data := null;
            begin
                result.next_id := 0;
                result.next_state := '{}'::jsonb;

                return result;
            end;
            $prototype$;
        "#;

        let (diagnostics, _span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 0);
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_if_expr(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE t1(a int, b int);
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.f1()
            RETURNS void
            LANGUAGE plpgsql
            AS $function$
            declare r t1 := (select t1 from t1 where a = 1);
            BEGIN
              if r.c is null or
                 true is false
                then -- there is bug - table t1 missing "c" column
                RAISE NOTICE 'c is null';
              end if;
            END;
            $function$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(
            span_texts[0].as_deref(),
            Some("if r.c is null or\n                 true is false\n                then")
        );
    }

    /// The checks use the connections of the pool. A function checked again with another body,
    /// on the same connection, reports the issues of the new body.
    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_redefined_on_one_connection(test_db: PgPool) {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_with((*test_db.connect_options()).clone())
            .await
            .expect("Failed to connect");
        let setup = "create extension if not exists plpgsql_check;";

        for variable in ["missing_one", "missing_two", "missing_one"] {
            let create_fn_sql = format!(
                "create function public.f1() returns int language plpgsql as $$
                begin
                    return {variable};
                end;
                $$;"
            );
            let (diagnostics, _) = run_plpgsql_check_test(&pool, setup, &create_fn_sql)
                .await
                .expect("Failed to run plpgsql_check test");
            assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
            let message = pgls_diagnostics::PrintDescription(&diagnostics[0]).to_string();
            assert!(message.contains(variable), "{message}");
        }
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_missing_var(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE t1(a int, b int);
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.f1()
            RETURNS void
            LANGUAGE plpgsql
            AS $function$
            BEGIN
                SELECT 1 from t1 where a = v_c;
            END;
            $function$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(span_texts[0].as_deref(), Some("v_c"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_missing_col_if_stmt(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE t1(a int, b int);
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.f1()
            RETURNS void
            LANGUAGE plpgsql
            AS $function$
            BEGIN
              if (select c from t1 where id = 1) is null then -- there is bug - table t1 missing "c" column
                RAISE NOTICE 'c is null';
              end if;
            END;
            $function$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(span_texts[0].as_deref(), Some("c"));
    }

    #[pgls_test_utils::db_test]
    async fn test_entire_body_broken(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE t1(a int, b int);
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.f1()
            RETURNS void
            LANGUAGE plpgsql
            AS 
            $function$    DECLRE r record; -- spelled declare wrong!
            BEGIN
                select * from t1;
            END;     $function$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        // the span is the token at the syntax error instead of the entire body
        assert_eq!(span_texts[0].as_deref(), Some("DECLRE"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE t1(a int, b int);
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.f1()
            RETURNS void
            LANGUAGE plpgsql
            AS $function$
            DECLARE r record;
            BEGIN
              FOR r IN SELECT * FROM t1
              LOOP
                RAISE NOTICE '%', r.c; -- there is bug - table t1 missing "c" column
              END LOOP;
            END;
            $function$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(span_texts[0].as_deref(), Some("RAISE NOTICE '%', r.c;"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_stacked_diagnostics(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;
        "#;

        let create_fn_sql = r#"
            create or replace function fxtest()
            returns void as $$
            declare
              v_sqlstate text;
              v_message text;
              v_context text;
            begin
              get stacked diagnostics
                v_sqlstate = returned_sqlstate,
                v_message = message_text,
                v_context = pg_exception_context;
            end;
            $$ language plpgsql;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(span_texts[0].as_deref(), Some("get stacked diagnostics"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_constant_refcursor(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;
            create table rc_test(a int);
        "#;

        let create_fn_sql = r#"
            create function return_constant_refcursor() returns refcursor as $$
            declare
                rc constant refcursor;
            begin
                open rc for select a from rc_test;
                return rc;
            end
            $$ language plpgsql;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(
            span_texts[0].as_deref(),
            Some("open rc for select a from rc_test;")
        );
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_constant_assignment(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            create procedure p1(a int, out b int)
            as $$
            begin
              b := a + 10;
            end;
            $$ language plpgsql;
        "#;

        let create_fn_sql = r#"
            create function f1()
            returns void as $$
            declare b constant int;
            begin
              call p1(10, b);
            end;
            $$ language plpgsql;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(span_texts[0].as_deref(), Some("call p1(10, b);"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_missing_procedure(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;
        "#;

        let create_fn_sql = r#"
            create function f1()
            returns void as $$
            declare b constant int;
            begin
              call p1(10, b);
            end;
            $$ language plpgsql;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert_eq!(span_texts[0].as_deref(), Some("p1"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_dml_in_stable_function(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;
            create table t1(a int, b int);
        "#;

        let create_fn_sql = r#"
            create function f1()
            returns void as $$
            begin
              if false then
                insert into t1 values(10,20);
                update t1 set a = 10;
                delete from t1;
              end if;
            end;
            $$ language plpgsql stable;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert_eq!(diagnostics.len(), 1);
        assert!(span_texts[0].is_some());

        assert_eq!(diagnostics[0].advices.code.as_deref(), Some("0A000"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_record_field_assignment(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            create function g1() returns table(a int, b int) as $$
            begin
              return query select 1, 2;
            end;
            $$ language plpgsql;
        "#;

        let create_fn_sql = r#"
            create or replace function f1()
            returns void as $$
            declare r record;
            begin
              for r in select * from g1()
              loop
                r.c := 20;
              end loop;
            end;
            $$ language plpgsql;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert!(span_texts[0].is_some());
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_trigger_basic(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE users(
                id serial primary key,
                name text not null,
                email text
            );

            CREATE OR REPLACE FUNCTION public.log_user_changes()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $function$
            BEGIN
                -- Intentional error: referencing non-existent column
                INSERT INTO audit_log(table_name, changed_id, old_email, new_email)
                VALUES ('users', NEW.id, OLD.email, NEW.email);
                RETURN NEW;
            END;
            $function$;

            CREATE TRIGGER trg_users_audit
                AFTER UPDATE ON users
                FOR EACH ROW
                EXECUTE FUNCTION public.log_user_changes();
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.log_user_changes()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $function$
            BEGIN
                -- Intentional error: referencing non-existent column
                INSERT INTO audit_log(table_name, changed_id, old_email, new_email)
                VALUES ('users', NEW.id, OLD.email, NEW.email);
                RETURN NEW;
            END;
            $function$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert!(diagnostics[0].advices.relation.is_some());
        assert_eq!(
            diagnostics[0].advices.relation.as_deref(),
            Some("public.users")
        );
        assert_eq!(span_texts[0].as_deref(), Some("audit_log"));
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_trigger_missing_column(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE products(
                id serial primary key,
                name text not null,
                price numeric(10,2)
            );

            CREATE OR REPLACE FUNCTION public.validate_product()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $function$
            BEGIN
                -- Error: referencing non-existent column
                IF NEW.category IS NULL THEN
                    RAISE EXCEPTION 'Category is required';
                END IF;
                RETURN NEW;
            END;
            $function$;

            CREATE TRIGGER trg_product_validation
                BEFORE INSERT OR UPDATE ON products
                FOR EACH ROW
                EXECUTE FUNCTION public.validate_product();
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.validate_product()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $function$
            BEGIN
                -- Error: referencing non-existent column
                IF NEW.category IS NULL THEN
                    RAISE EXCEPTION 'Category is required';
                END IF;
                RETURN NEW;
            END;
            $function$;
        "#;

        let (diagnostics, span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(matches!(
            diagnostics[0].severity,
            pgls_diagnostics::Severity::Error
        ));
        assert!(span_texts[0].as_deref().unwrap().contains("category"));
        assert_eq!(
            diagnostics[0].advices.relation.as_deref(),
            Some("public.products")
        );
    }

    #[pgls_test_utils::db_test]
    async fn test_plpgsql_check_trigger_multiple_tables(test_db: PgPool) {
        let setup = r#"
            create extension if not exists plpgsql_check;

            CREATE TABLE table_a(
                id serial primary key,
                name text
            );

            CREATE TABLE table_b(
                id serial primary key,
                description text
            );

            CREATE OR REPLACE FUNCTION public.generic_audit()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $function$
            BEGIN
                -- Error: referencing column that doesn't exist in both tables
                INSERT INTO audit_log(table_name, record_id, old_status)
                VALUES (TG_TABLE_NAME, NEW.id, OLD.status);
                RETURN NEW;
            END;
            $function$;

            CREATE TRIGGER trg_audit_a
                AFTER UPDATE ON table_a
                FOR EACH ROW
                EXECUTE FUNCTION public.generic_audit();

            CREATE TRIGGER trg_audit_b
                AFTER UPDATE ON table_b
                FOR EACH ROW
                EXECUTE FUNCTION public.generic_audit();
        "#;

        let create_fn_sql = r#"
            CREATE OR REPLACE FUNCTION public.generic_audit()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $function$
            BEGIN
                -- Error: referencing column that doesn't exist in both tables
                INSERT INTO audit_log(table_name, record_id, old_status)
                VALUES (TG_TABLE_NAME, NEW.id, OLD.status);
                RETURN NEW;
            END;
            $function$;
        "#;

        let (diagnostics, _span_texts) = run_plpgsql_check_test(&test_db, setup, create_fn_sql)
            .await
            .expect("Failed to run plpgsql_check test");

        assert!(!diagnostics.is_empty());
        assert!(diagnostics.len() >= 2);

        let relations: Vec<_> = diagnostics
            .iter()
            .filter_map(|d| d.advices.relation.as_ref())
            .collect();
        assert!(relations.contains(&&"public.table_a".to_string()));
        assert!(relations.contains(&&"public.table_b".to_string()));
    }
}

#[cfg(test)]
mod result_tests {
    use super::*;

    #[test]
    fn deserializes_issue_detail_and_hint() {
        let result: PlPgSqlCheckResult = serde_json::from_str(
            r#"{
                "function": "public.f1",
                "issues": [{
                    "level": "error",
                    "message": "cannot cast type text to uuid",
                    "detail": "Input has to match the uuid format.",
                    "hint": "Validate the input first.",
                    "sqlState": "42846"
                }]
            }"#,
        )
        .expect("plpgsql_check JSON should parse");

        let issue = &result.issues[0];
        assert_eq!(
            issue.detail.as_deref(),
            Some("Input has to match the uuid format.")
        );
        assert_eq!(issue.hint.as_deref(), Some("Validate the input first."));
    }
}
