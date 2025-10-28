mod diagnostics;

pub use diagnostics::PlPgSqlCheckDiagnostic;
use diagnostics::create_diagnostics_from_check_result;
use pgls_query::protobuf::CreateFunctionStmt;
use regex::Regex;
use serde::Deserialize;
pub use sqlx::postgres::PgSeverity;
use sqlx::{Acquire, PgPool, Postgres, Transaction};

#[derive(Debug)]
pub struct PlPgSqlCheckParams<'a> {
    pub conn: &'a PgPool,
    pub sql: &'a str,
    pub ast: &'a pgls_query::NodeEnum,
    pub schema_cache: &'a pgls_schema_cache::SchemaCache,
}

#[derive(Debug, Deserialize)]
pub struct PlpgSqlCheckResult {
    pub function: String,
    pub issues: Vec<PlpgSqlCheckIssue>,
}

#[derive(Debug, Deserialize)]
pub struct PlpgSqlCheckIssue {
    pub level: String,
    pub message: String,
    pub statement: Option<Statement>,
    pub query: Option<Query>,
    #[serde(rename = "sqlState")]
    pub sql_state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Statement {
    #[serde(rename = "lineNumber")]
    pub line_number: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct Query {
    pub position: String,
    pub text: String,
}

/// check if the given node is a plpgsql function that should be checked
fn should_check_function<'a>(
    ast: &'a pgls_query::NodeEnum,
    schema_cache: &pgls_schema_cache::SchemaCache,
) -> Option<&'a CreateFunctionStmt> {
    let create_fn = match ast {
        pgls_query::NodeEnum::CreateFunctionStmt(stmt) => stmt,
        _ => return None,
    };

    if pgls_query_ext::utils::find_option_value(create_fn, "language")
        != Some("plpgsql".to_string())
    {
        return None;
    }

    if !schema_cache
        .extensions
        .iter()
        .any(|e| e.name == "plpgsql_check")
    {
        return None;
    }

    Some(create_fn)
}

/// check if a function is a trigger function
fn is_trigger_function(create_fn: &CreateFunctionStmt) -> bool {
    create_fn
        .return_type
        .as_ref()
        .map(|n| {
            matches!(
                pgls_query_ext::utils::parse_name(&n.names),
                Some((None, name)) if name == "trigger"
            )
        })
        .unwrap_or(false)
}

/// build the function identifier string used by plpgsql_check
fn build_function_identifier(
    create_fn: &CreateFunctionStmt,
    fn_schema: &Option<String>,
    fn_name: &str,
) -> String {
    let args = create_fn
        .parameters
        .iter()
        .filter_map(|arg| {
            let node = match &arg.node {
                Some(pgls_query::NodeEnum::FunctionParameter(n)) => n,
                _ => return None,
            };
            let type_name_node = node.arg_type.as_ref()?;
            let type_name = match pgls_query_ext::utils::parse_name(&type_name_node.names) {
                Some((schema, name)) => match schema {
                    Some(s) => format!("{s}.{name}"),
                    None => name,
                },
                None => return None,
            };

            if !type_name_node.array_bounds.is_empty() {
                Some(format!("{type_name}[]"))
            } else {
                Some(type_name)
            }
        })
        .collect::<Vec<_>>();

    let fn_qualified_name = match fn_schema {
        Some(schema) => format!("{schema}.{fn_name}"),
        None => fn_name.to_string(),
    };

    if args.is_empty() {
        fn_qualified_name
    } else {
        format!("{}({})", fn_qualified_name, args.join(", "))
    }
}

pub async fn check_plpgsql(
    params: PlPgSqlCheckParams<'_>,
) -> Result<Vec<PlPgSqlCheckDiagnostic>, sqlx::Error> {
    let create_fn = match should_check_function(params.ast, params.schema_cache) {
        Some(stmt) => stmt,
        None => return Ok(vec![]),
    };

    let (fn_schema, fn_name) = match pgls_query_ext::utils::parse_name(&create_fn.funcname) {
        Some(n) => n,
        None => return Ok(vec![]),
    };

    let fn_identifier = build_function_identifier(create_fn, &fn_schema, &fn_name);

    let fn_body = pgls_query_ext::utils::find_option_value(create_fn, "as")
        .ok_or_else(|| sqlx::Error::Protocol("Failed to find function body".to_string()))?;
    let offset = params
        .sql
        .find(&fn_body)
        .ok_or_else(|| sqlx::Error::Protocol("Failed to find function body in SQL".to_string()))?;
    let is_trigger = is_trigger_function(create_fn);

    let mut conn = params.conn.acquire().await?;
    conn.close_on_drop();

    let mut tx: Transaction<'_, Postgres> = conn.begin().await?;

    // disable function body checking to rely on plpgsql_check
    sqlx::query("SET LOCAL check_function_bodies = off")
        .execute(&mut *tx)
        .await?;

    // make sure we run with "or replace"
    let sql_with_replace = if !create_fn.replace {
        let re = Regex::new(r"(?i)\bCREATE\s+FUNCTION\b").unwrap();
        re.replace(params.sql, "CREATE OR REPLACE FUNCTION")
            .to_string()
    } else {
        params.sql.to_string()
    };

    // create the function - this should always succeed
    sqlx::query(&sql_with_replace).execute(&mut *tx).await?;

    // run plpgsql_check and collect results with their relations
    let results_with_relations: Vec<(String, Option<String>)> = if is_trigger {
        let mut results = Vec::new();

        for trigger in params.schema_cache.triggers.iter() {
            if trigger.proc_name == fn_name
                && (fn_schema.is_none() || fn_schema.as_deref() == Some(&trigger.proc_schema))
            {
                let relation = format!("{}.{}", trigger.table_schema, trigger.table_name);

                let result: Option<String> = sqlx::query_scalar(&format!(
                    "select plpgsql_check_function('{fn_identifier}', '{relation}', format := 'json')"
                ))
                .fetch_optional(&mut *tx)
                .await?
                .flatten();

                if let Some(result) = result {
                    results.push((result, Some(relation)));
                }
            }
        }

        results
    } else {
        let result: Option<String> = sqlx::query_scalar(&format!(
            "select plpgsql_check_function('{fn_identifier}', format := 'json')"
        ))
        .fetch_optional(&mut *tx)
        .await?
        .flatten();

        if let Some(result) = result {
            vec![(result, None)]
        } else {
            vec![]
        }
    };

    tx.rollback().await?;

    // Parse results and create diagnostics
    let mut diagnostics = Vec::new();
    for (result_json, relation) in results_with_relations {
        let check_result: PlpgSqlCheckResult = serde_json::from_str(&result_json).map_err(|e| {
            sqlx::Error::Protocol(format!("Failed to parse plpgsql_check result: {e}"))
        })?;

        let mut result_diagnostics =
            create_diagnostics_from_check_result(&check_result, &fn_body, offset, relation);
        diagnostics.append(&mut result_diagnostics);
    }

    Ok(diagnostics)
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
        let schema_cache = pgls_schema_cache::SchemaCache::load(test_db).await?;

        let diagnostics = super::check_plpgsql(super::PlPgSqlCheckParams {
            conn: test_db,
            sql: create_fn_sql,
            ast: &ast,
            schema_cache: &schema_cache,
        })
        .await?;

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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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
        assert_eq!(
            span_texts[0].as_deref(),
            // the span starts at the keyword and omits the whitespace before/after $function$
            Some(
                r#"DECLRE r record; -- spelled declare wrong!
            BEGIN
                select * from t1;
            END;"#
            )
        );
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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
