use pgt_console::fmt::{Formatter, HTML};
use pgt_diagnostics::Diagnostic;
use pgt_typecheck::{IdentifierType, TypecheckParams, TypedIdentifier, check_sql};
use sqlx::{Executor, PgPool};
use std::fmt::Write;

struct TestSetup<'a> {
    name: &'a str,
    query: &'a str,
    setup: Option<&'a str>,
    test_db: &'a PgPool,
    typed_identifiers: Vec<TypedIdentifier>,
}

impl TestSetup<'_> {
    async fn test(self) {
        if let Some(setup) = self.setup {
            self.test_db
                .execute(setup)
                .await
                .expect("Failed to setup test selfbase");
        }

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgt_treesitter_grammar::LANGUAGE.into())
            .expect("Error loading sql language");

        let schema_cache = pgt_schema_cache::SchemaCache::load(self.test_db)
            .await
            .expect("Failed to load Schema Cache");

        let root = pgt_query::parse(self.query)
            .unwrap()
            .into_root()
            .expect("Failed to parse query");
        let tree = parser.parse(self.query, None).unwrap();

        let result = check_sql(TypecheckParams {
            conn: self.test_db,
            sql: self.query,
            ast: &root,
            tree: &tree,
            schema_cache: &schema_cache,
            identifiers: self.typed_identifiers,
            search_path_patterns: vec![],
        })
        .await;

        assert!(
            result.is_ok(),
            "Got Typechecking error: {}",
            result.unwrap_err()
        );

        let maybe_diagnostic = result.unwrap();

        let content = match maybe_diagnostic {
            Some(d) => {
                let mut result = String::new();

                if let Some(span) = d.location().span {
                    for (idx, c) in self.query.char_indices() {
                        if pgt_text_size::TextSize::new(idx.try_into().unwrap()) == span.start() {
                            result.push_str("~~~");
                        }
                        if pgt_text_size::TextSize::new(idx.try_into().unwrap()) == span.end() {
                            result.push_str("~~~");
                        }
                        result.push(c);
                    }
                } else {
                    result.push_str("~~~");
                    result.push_str(self.query);
                    result.push_str("~~~");
                }

                writeln!(&mut result).unwrap();
                writeln!(&mut result).unwrap();

                let mut msg_content = vec![];
                let mut writer = HTML::new(&mut msg_content);
                let mut formatter = Formatter::new(&mut writer);
                d.message(&mut formatter).unwrap();

                result.push_str(String::from_utf8(msg_content).unwrap().as_str());

                result
            }
            None => String::from("No Diagnostic"),
        };

        insta::with_settings!({
            prepend_module_to_snapshot => false,
        }, {
            insta::assert_snapshot!(self.name, content);

        });
    }
}

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
async fn invalid_column(test_db: PgPool) {
    TestSetup {
        name: "invalid_column",
        query: "select id, unknown from contacts;",
        setup: Some(
            r#"
        create table public.contacts (
            id serial primary key,
            name varchar(255) not null,
            is_vegetarian bool default false,
            middle_name varchar(255)
        );
    "#,
        ),
        test_db: &test_db,
        typed_identifiers: vec![],
    }
    .test()
    .await;
}

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
async fn invalid_type_in_function(test_db: PgPool) {
    // create or replace function clean_up(uid uuid)
    // returns void
    // language sql
    // as $$
    //     delete from public.contacts where id = uid;
    // $$;

    let setup = r#"
        create table public.contacts (
            id serial primary key,
            name text not null,
            is_vegetarian bool default false,
            middle_name varchar(255)
        );
    "#;

    /* NOTE: The replaced type default value is *longer* than the param name. */
    TestSetup {
        name: "invalid_type_in_function_longer_default",
        setup: Some(setup),
        query: r#"delete from public.contacts where id = uid;"#,
        test_db: &test_db,
        typed_identifiers: vec![TypedIdentifier {
            path: "clean_up".to_string(),
            name: Some("uid".to_string()),
            type_: IdentifierType {
                schema: None,
                name: "uuid".to_string(),
                is_array: false,
            },
        }],
    }
    .test()
    .await;

    /* NOTE: The replaced type default value is *shorter* than the param name. */
    TestSetup {
        name: "invalid_type_in_function_shorter_default",
        setup: None,
        query: r#"delete from public.contacts where id = contact_name;"#,
        test_db: &test_db,
        typed_identifiers: vec![TypedIdentifier {
            path: "clean_up".to_string(),
            name: Some("contact_name".to_string()),
            type_: IdentifierType {
                schema: None,
                name: "text".to_string(),
                is_array: false,
            },
        }],
    }
    .test()
    .await;
}
