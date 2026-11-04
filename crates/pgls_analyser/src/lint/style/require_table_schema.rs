use crate::fix::StatementTokens;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{Node, ObjectType, RangeVar, Token};
use pgls_query::{NodeEnum, NodeRef};
use pgls_text_size::TextRange;

/// The `relpersistence` of temporary tables, which always live in `pg_temp`.
const RELPERSISTENCE_TEMP: &str = "t";

declare_lint_rule! {
    /// Table names in DDL should be schema-qualified.
    ///
    /// Without a schema, Postgres resolves the table through the `search_path`, which depends on
    /// the session that runs the migration. This makes it ambiguous which table a statement acts
    /// on. Temporary tables, including the ones created earlier in the file, are not reported.
    ///
    /// The rule checks the statements that create, change, or drop a table:
    /// `CREATE [FOREIGN] TABLE`, `CREATE TABLE AS`, `SELECT INTO`, `ALTER [FOREIGN] TABLE`
    /// (including `RENAME` and `SET SCHEMA`), `CREATE INDEX`, and `DROP [FOREIGN] TABLE`. It also
    /// checks the tables these statements reference by name, like `REFERENCES`, `PARTITION OF`,
    /// `INHERITS`, `LIKE`, and `ATTACH PARTITION`. Queries, like the one of a view, are not
    /// checked.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table posts (id bigint);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter table posts add column title text;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table public.posts (id bigint);
    /// alter table public.posts add column title text;
    /// drop table public.posts;
    /// create temp table scratch (id bigint);
    /// ```
    ///
    pub RequireTableSchema {
        version: "next",
        name: "requireTableSchema",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("require-table-schema")],
    }
}

impl LinterRule for RequireTableSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        let table = |relation: &RangeVar| Table::new(relation, tokens);
        let tables: Vec<Table> = match ctx.stmt() {
            // Every relation in these statements is a table: the one they act on, and the ones
            // they reference with REFERENCES, PARTITION OF, INHERITS, LIKE or ATTACH PARTITION.
            NodeEnum::CreateStmt(_)
            | NodeEnum::CreateForeignTableStmt(_)
            | NodeEnum::IndexStmt(_) => range_vars(ctx.stmt(), tokens),
            NodeEnum::AlterTableStmt(stmt)
                if matches!(
                    stmt.objtype(),
                    ObjectType::ObjectTable | ObjectType::ObjectForeignTable
                ) =>
            {
                range_vars(ctx.stmt(), tokens)
            }
            NodeEnum::CreateTableAsStmt(stmt) if stmt.objtype() == ObjectType::ObjectTable => stmt
                .into
                .iter()
                .filter_map(|into| into.rel.as_ref())
                .map(table)
                .collect(),
            NodeEnum::SelectStmt(stmt) => stmt
                .into_clause
                .iter()
                .filter_map(|into| into.rel.as_ref())
                .map(table)
                .collect(),
            NodeEnum::RenameStmt(stmt)
                if is_table(stmt.rename_type()) || is_table(stmt.relation_type()) =>
            {
                stmt.relation.iter().map(table).collect()
            }
            NodeEnum::AlterObjectSchemaStmt(stmt) if is_table(stmt.object_type()) => {
                stmt.relation.iter().map(table).collect()
            }
            NodeEnum::DropStmt(stmt) if is_table(stmt.remove_type()) => {
                dropped_tables(&stmt.objects, tokens)
            }
            _ => vec![],
        };

        let temporary = temporary_tables(ctx.file_context().previous_stmts());
        tables
            .into_iter()
            .filter(|table| {
                table.schema.is_none()
                    && !table.temporary
                    && !temporary.contains(&table.name.as_str())
            })
            .map(|table| {
                let name = table.name;
                LinterDiagnostic::new(
                    rule_category!(),
                    table.span,
                    markup! {
                        "Table "<Emphasis>{name}</Emphasis>" is not schema-qualified."
                    },
                )
                .detail(
                    None,
                    "Without a schema, the table is resolved through the search_path of the session that runs the statement.",
                )
                .note(markup! {
                    "Specify the schema, e.g. "<Emphasis>"public."{name}</Emphasis>"."
                })
            })
            .collect()
    }
}

/// A table name in a statement.
struct Table {
    name: String,
    schema: Option<String>,
    temporary: bool,
    span: Option<TextRange>,
}

impl Table {
    fn new(relation: &RangeVar, tokens: Option<StatementTokens>) -> Self {
        Self {
            name: relation.relname.clone(),
            schema: (!relation.schemaname.is_empty()).then(|| relation.schemaname.clone()),
            temporary: relation.relpersistence == RELPERSISTENCE_TEMP,
            span: tokens.and_then(|tokens| Some(tokens.range(tokens.at(relation.location)?))),
        }
    }
}

fn is_table(object_type: ObjectType) -> bool {
    matches!(
        object_type,
        ObjectType::ObjectTable | ObjectType::ObjectForeignTable
    )
}

fn range_vars(stmt: &NodeEnum, tokens: Option<StatementTokens>) -> Vec<Table> {
    stmt.iter()
        .filter_map(|node| match node {
            NodeRef::RangeVar(relation) => Some(Table::new(relation, tokens)),
            _ => None,
        })
        .collect()
}

/// The tables of a `DROP TABLE`. Their names have no locations, but they are listed in order,
/// separated by commas, after `TABLE` and `IF EXISTS`.
fn dropped_tables(objects: &[Node], tokens: Option<StatementTokens>) -> Vec<Table> {
    let mut spans = tokens.map(dropped_names).unwrap_or_default().into_iter();
    objects
        .iter()
        .map(|object| {
            let span = spans.next();
            let mut names: Vec<String> = match &object.node {
                Some(NodeEnum::List(list)) => list
                    .items
                    .iter()
                    .filter_map(|item| match &item.node {
                        Some(NodeEnum::String(name)) => Some(name.sval.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            };
            let name = names.pop().unwrap_or_default();
            Table {
                name,
                schema: names.pop(),
                temporary: false,
                span,
            }
        })
        .collect()
}

/// The ranges of the comma-separated names after `DROP [FOREIGN] TABLE [IF EXISTS]`.
fn dropped_names(tokens: StatementTokens) -> Vec<TextRange> {
    let Some(table) = tokens.find(Token::Table) else {
        return vec![];
    };
    let mut index = table + 1;
    if tokens.kind(index) == Some(Token::IfP) {
        index += 2;
    }
    let mut names = Vec::new();
    let mut current: Option<TextRange> = None;
    while let Some(kind) = tokens.kind(index) {
        match kind {
            Token::Ascii44 => names.extend(current.take()),
            Token::Ascii59 | Token::Cascade | Token::Restrict => break,
            Token::SqlComment | Token::CComment => {}
            _ => {
                let range = tokens.range(index);
                current = Some(current.map_or(range, |current| current.cover(range)));
            }
        }
        index += 1;
    }
    names.extend(current);
    names
}

/// The unqualified names of the temporary tables created by earlier statements of the file.
/// An unqualified name refers to them, since `pg_temp` comes first in the search path.
fn temporary_tables(previous: &[NodeEnum]) -> Vec<&str> {
    let mut tables = Vec::new();
    for stmt in previous {
        let created = match stmt {
            NodeEnum::CreateStmt(stmt) => stmt.relation.as_ref(),
            NodeEnum::CreateTableAsStmt(stmt) => {
                stmt.into.as_ref().and_then(|into| into.rel.as_ref())
            }
            NodeEnum::SelectStmt(stmt) => {
                stmt.into_clause.as_ref().and_then(|into| into.rel.as_ref())
            }
            _ => None,
        };
        if let Some(relation) = created
            && (relation.relpersistence == RELPERSISTENCE_TEMP || relation.schemaname == "pg_temp")
        {
            tables.push(relation.relname.as_str());
        }
        if let NodeEnum::DropStmt(stmt) = stmt
            && is_table(stmt.remove_type())
        {
            for table in dropped_tables(&stmt.objects, None) {
                if table.schema.is_none_or(|schema| schema == "pg_temp") {
                    tables.retain(|name| *name != table.name);
                }
            }
        }
    }
    tables
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireTableSchema", sql)
    }

    #[test]
    fn points_at_the_table_name() {
        assert_eq!(
            spans("alter table \"Posts\" add column title text"),
            ["\"Posts\""]
        );
        assert_eq!(
            spans("create table public.posts (user_id int references users (id))"),
            ["users"]
        );
        assert_eq!(
            spans("drop table if exists public.a, b, \"C\" /* c */ cascade"),
            ["b", "\"C\""]
        );
    }
}
