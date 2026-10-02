//! Formats every statement Postgres accepted in its own regression suite and checks that the output
//! parses back to the same AST.

use std::panic::{AssertUnwindSafe, catch_unwind};

use pgls_postgres_regress::Verdict;
use pgls_pretty_print::{
    FormatConfig,
    emitter::EventEmitter,
    nodes::emit_node_enum,
    normalize::normalize_ast,
    renderer::{RenderConfig, Renderer},
};
use rayon::prelude::*;

const LINE_WIDTHS: [usize; 2] = [80, 100];

#[test]
fn formatting_preserves_every_statement_postgres_accepted() {
    let parser_major = pgls_query::parse("select 1")
        .expect("parse select 1")
        .protobuf
        .version as u32
        / 10000;
    // Newer versions may have syntax the parser doesn't know yet.
    let files = pgls_postgres_regress::versions()
        .into_iter()
        .filter(|x| x.major <= parser_major)
        .flat_map(|version| {
            let major = version.major;
            version.files().into_iter().map(move |file| (major, file))
        })
        .collect::<Vec<_>>();
    let results = files
        .par_iter()
        .flat_map_iter(|(major, file)| {
            file.statements
                .iter()
                .filter(|statement| statement.verdict == Verdict::Accepted)
                .map(move |statement| {
                    let sql = file.sql(statement);
                    round_trip(sql).map_err(|e| {
                        let (line, col) = file.line_col(statement);
                        format!("{major}/{}.sql:{line}:{col}: {e}\n  {sql}", file.name)
                    })
                })
        })
        .collect::<Vec<_>>();
    let checked = results.len();
    let failures = results
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    assert!(checked > 0, "no accepted statements to check");
    assert!(
        failures.is_empty(),
        "{} of {checked} accepted statements don't survive formatting:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn round_trip(sql: &str) -> Result<(), String> {
    // An empty statement (a lone `;`) has nothing to format.
    let Some(ast) = pgls_query::parse(sql)
        .map_err(|e| format!("input doesn't parse: {e}"))?
        .into_root()
    else {
        return Ok(());
    };
    let mut expected = ast.clone();
    normalize_ast(&mut expected);
    for max_line_length in LINE_WIDTHS {
        let output = catch_unwind(AssertUnwindSafe(|| format(&ast, max_line_length)))
            .map_err(|_| format!("formatter panicked at width {max_line_length}"))?;
        let mut formatted = pgls_query::parse(&output)
            .map_err(|e| format!("output doesn't parse at width {max_line_length}: {e}\n{output}"))?
            .into_root()
            .ok_or("output has no statement")?;
        normalize_ast(&mut formatted);
        if formatted != expected {
            return Err(format!(
                "output has a different AST at width {max_line_length}\n{output}"
            ));
        }
    }
    Ok(())
}

fn format(ast: &pgls_query::NodeEnum, max_line_length: usize) -> String {
    let config = FormatConfig::default();
    let mut emitter = EventEmitter::new(config.clone());
    emit_node_enum(ast, &mut emitter);
    let mut output = String::new();
    let config = RenderConfig {
        max_line_length,
        ..RenderConfig::from(config)
    };
    Renderer::new(&mut output, config)
        .render(emitter.events)
        .expect("render");
    output
}
