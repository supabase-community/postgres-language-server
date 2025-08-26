use camino::Utf8Path;
use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};

use pgt_pretty_print::{
    emitter::{EventEmitter, ToTokens},
    renderer::{IndentStyle, RenderConfig, Renderer},
};

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/",
    glob: "*.sql",
)]
fn test_formatter(fixture: Fixture<&str>) {
    let content = fixture.content();
    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let input_file = absolute_fixture_path;
    let test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    // extract line length from filename (e.g., "simple_select_80" -> 80)
    let max_line_length = test_name
        .split('_')
        .next_back()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(80);

    let parsed = pgt_query::parse(content).expect("Failed to parse SQL");
    let mut ast = parsed.into_root().expect("No root node found");

    let mut emitter = EventEmitter::new();
    ast.to_tokens(&mut emitter);

    let mut output = String::new();
    let config = RenderConfig {
        max_line_length,
        indent_size: 2,
        indent_style: IndentStyle::Spaces,
    };
    let mut renderer = Renderer::new(&mut output, config);
    renderer.render(emitter.events).expect("Failed to render");

    let parsed_output = pgt_query::parse(&output).expect("Failed to parse SQL");
    let mut parsed_ast = parsed_output.into_root().expect("No root node found");

    // the location fields are now different in the two ASTs
    clear_location(&mut parsed_ast);
    clear_location(&mut ast);

    assert_eq!(ast, parsed_ast);

    with_settings!({
      omit_expression => true,
      input_file => input_file,
    }, {
      assert_snapshot!(test_name, output);
    });
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/data/",
    glob: "*.sql",
)]
fn validate_test_data(fixture: Fixture<&str>) {
    let content = fixture.content();
    let absolute_fixture_path = Utf8Path::new(fixture.path());
    let test_name = absolute_fixture_path
        .file_name()
        .and_then(|x| x.strip_suffix(".sql"))
        .unwrap();

    let result = pgt_query::parse(content);

    if let Ok(res) = result.as_ref() {
        assert!(res.root().is_some(), "should have a single root node");
    }

    assert!(result.is_ok(), "should be valid SQL");
}

fn clear_location(node: &mut pgt_query::NodeEnum) {
    unsafe {
        node.iter_mut().for_each(|n| match n {
            pgt_query::NodeMut::ColumnRef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::ParamRef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::AExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::TypeCast(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::CollateClause(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::FuncCall(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::AArrayExpr(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::ResTarget(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::SortBy(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::WindowDef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::TypeName(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::ColumnDef(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::DefElem(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::XmlSerialize(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::AConst(n) => {
                (*n).location = 0;
            }
            pgt_query::NodeMut::RangeVar(n) => {
                (*n).location = 0;
            }
            _ => {}
        });
    }
}
