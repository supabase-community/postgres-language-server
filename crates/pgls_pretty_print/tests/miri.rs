//! Runs `normalize_ast` under Miri, which checks the only raw-pointer code in Rust that the
//! formatter's safety check runs on: `pgls_query`'s `NodeMut` and its iterator hand out `*mut`
//! pointers into the tree, and `normalize_ast` flattens the `BoolExpr`s through them, children
//! before parents. The comparison of 2P3 replaces `NodeMut`, and this test with it.
//!
//! Miri can't call libpg_query, so the statements come parsed: `data/miri/statements.pb` is the
//! protobuf of `data/miri/statements.sql`, a statement of each file of Postgres' regression suite,
//! which prost decodes in Rust. `protobuf_is_current` checks it, and rewrites it with
//! `UPDATE_MIRI_FIXTURE=1`, e.g. after an upgrade of libpg_query.
//!
//! The nightly workflow runs it with
//! `MIRIFLAGS=-Zmiri-tree-borrows cargo +nightly miri test -p pgls_pretty_print --test miri`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on unexpected data"
)]

use pgls_pretty_print::normalize_ast;
use pgls_query::protobuf::ParseResult;
use prost::Message;

const SQL: &str = include_str!("data/miri/statements.sql");
const PROTOBUF: &[u8] = include_bytes!("data/miri/statements.pb");

#[test]
fn normalize_ast_under_miri() {
    let parsed = ParseResult::decode(PROTOBUF).expect("decode the fixture");
    assert!(!parsed.stmts.is_empty(), "no statements in the fixture");
    for statement in parsed.stmts {
        let Some(mut node) = statement.stmt.and_then(|node| node.node) else {
            continue;
        };
        normalize_ast(&mut node);
        let normalized = node.clone();
        normalize_ast(&mut node);
        assert!(node == normalized, "normalize_ast isn't idempotent");
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls libpg_query")]
fn protobuf_is_current() {
    let encoded = pgls_query::parse(SQL)
        .expect("the fixture parses")
        .protobuf
        .encode_to_vec();
    if std::env::var_os("UPDATE_MIRI_FIXTURE").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/miri/statements.pb");
        std::fs::write(path, &encoded).expect("write the fixture");
        return;
    }
    assert!(
        encoded == PROTOBUF,
        "tests/data/miri/statements.pb isn't the protobuf of statements.sql; run the test with \
         UPDATE_MIRI_FIXTURE=1"
    );
}
