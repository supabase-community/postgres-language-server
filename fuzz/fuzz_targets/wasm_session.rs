//! Random sessions against the WASM language server (`pgls_wasm::LspHandler`): edits, undos,
//! completions, hovers and code actions anywhere, reopens, configuration changes and raw
//! messages. Every message gets JSON back, no request fails with an internal error, published
//! diagnostics lie inside their text, and at the end the diagnostics equal those of a fresh
//! handler that only opened the final text.
//!
//! `pgls_workspace`'s document test compares every version of a document with a fresh one; this
//! target covers the handler on top: its copies of the documents and what it publishes.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use pgls_wasm::LspHandler;
use serde_json::{Value, json};

/// The longest session.
const OPS_MAX: usize = 40;

/// The longest text; longer ones mostly cost time.
const TEXT_LEN_MAX: usize = 4000;

/// JSON-RPC's code for an internal error.
const INTERNAL_ERROR: i64 = -32603;

const URI: &str = "file:///a.sql";

const SNIPPETS: &[&str] = &[
    "select * from t;",
    "create table t (id int, name text);",
    "\n",
    "\n\n",
    "\r\n",
    "\r",
    ";",
    "select ",
    "from ",
    "t",
    "'",
    "$$",
    "-- c\n",
    "/* x */",
    "é",
    "🐘",
    "begin;",
    "commit;",
    "create function f() returns int language sql as $$ select 1 $$;",
    "alter table t add column c int;",
    "insert into t values (1, 'x');",
    "create schema s create table x (a int);",
    "with x as (select 1) select * from x;",
    "drop table t;",
    "update t set id = 1;",
    "(",
    ")",
    ",",
    "select 1 + ",
    "delete from t where id = 1;",
    "create index on t (id);",
    "\"T\"",
    "alter table t alter column id type bigint;",
    "select name from t where id = $1;",
];

#[derive(Arbitrary, Debug)]
enum Op {
    Insert { at: u16, snippet: u8 },
    InsertRaw { at: u16, text: String },
    Delete { at: u16, len: u8 },
    /// Undoes the last edit.
    Undo,
    Complete { at: u16 },
    Hover { at: u16 },
    CodeAction { at: u16 },
    /// A completion and a hover at a position that may be past the end.
    RawPosition { line: u8, character: u8 },
    Reopen,
    Configure { section_markers: bool },
    /// A message of any content, or, if it's JSON, the parameters of a request.
    Raw(String),
}

fuzz_target!(|ops: Vec<Op>| {
    if ops.len() > OPS_MAX {
        return;
    }
    let mut handler = Handler::new();
    let mut text = Text(String::from(
        "create table t (id int, name text);\nselect * from t;\n",
    ));
    let mut undo = Vec::new();
    let mut version = 1;
    let mut settings = json!({});
    handler.open(&text, version);
    for op in ops {
        let before = text.clone();
        let edited = match op {
            Op::Insert { at, snippet } => {
                let at = text.byte_at(at);
                text.0
                    .insert_str(at, SNIPPETS[usize::from(snippet) % SNIPPETS.len()]);
                true
            }
            Op::InsertRaw { at, text: raw } => {
                let at = text.byte_at(at);
                text.0.insert_str(at, &raw);
                true
            }
            Op::Delete { at, len } => {
                let start = text.byte_at(at);
                let end = (start..=(start + usize::from(len)).min(text.0.len()))
                    .rev()
                    .find(|&end| text.0.is_char_boundary(end))
                    .unwrap_or(start);
                text.0.replace_range(start..end, "");
                true
            }
            Op::Undo => match undo.pop() {
                Some(previous) => {
                    text = previous;
                    version += 1;
                    handler.change(&text, version);
                    continue;
                }
                None => false,
            },
            Op::Complete { at } | Op::Hover { at } | Op::CodeAction { at } => {
                let position = text.position(text.byte_at(at));
                let (method, params) = match op {
                    Op::Complete { .. } => (
                        "textDocument/completion",
                        json!({"textDocument": {"uri": URI}, "position": position}),
                    ),
                    Op::Hover { .. } => (
                        "textDocument/hover",
                        json!({"textDocument": {"uri": URI}, "position": position}),
                    ),
                    _ => (
                        "textDocument/codeAction",
                        json!({
                            "textDocument": {"uri": URI},
                            "range": {"start": position, "end": position},
                            "context": {"diagnostics": []},
                        }),
                    ),
                };
                handler.request(method, params);
                false
            }
            Op::RawPosition { line, character } => {
                let position = json!({"line": line, "character": character});
                let params = json!({"textDocument": {"uri": URI}, "position": position});
                handler.request("textDocument/completion", params.clone());
                handler.request("textDocument/hover", params);
                false
            }
            Op::Reopen => {
                version += 1;
                handler.open(&text, version);
                false
            }
            Op::Configure { section_markers } => {
                settings = if section_markers {
                    json!({"migrations": {"sectionMarkers": "dbmate"}})
                } else {
                    json!({})
                };
                handler.configure(&settings);
                false
            }
            Op::Raw(message) => {
                // A message that is JSON could change what the handler knows, e.g. the schema,
                // which the fresh handler wouldn't, so it only goes in as parameters.
                match serde_json::from_str::<Value>(&message) {
                    Ok(params) => {
                        handler.request("textDocument/hover", params);
                    }
                    Err(_) => {
                        handler.handle(&message);
                    }
                }
                false
            }
        };
        if !edited {
            continue;
        }
        if text.0.len() > TEXT_LEN_MAX {
            text = before;
            continue;
        }
        undo.push(before);
        version += 1;
        let diagnostics = handler.change(&text, version);
        text.check(&diagnostics);
    }

    // The final diagnostics, after a change that keeps the text.
    version += 1;
    let edited = handler.change(&text, version);
    let mut fresh = Handler::new();
    fresh.configure(&settings);
    let fresh = fresh.open(&text, 1);
    assert!(
        edited == fresh,
        "wasm_session: the diagnostics after the edits differ from a fresh handler's\nedited: \
         {edited}\nfresh: {fresh}\ntext: {:?}",
        text.0
    );
});

/// A handler that has been initialized.
struct Handler(LspHandler);

impl Handler {
    fn new() -> Self {
        let mut handler = Self(LspHandler::new());
        handler.request("initialize", json!({"capabilities": {}}));
        handler.send(&json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
        handler
    }

    /// Handles a message, and returns what the handler sent back, which is always JSON.
    fn handle(&mut self, message: &str) -> Vec<Value> {
        let out = self.0.handle_message(message);
        let Ok(messages) = serde_json::from_str::<Vec<Value>>(&out) else {
            panic!("wasm_session: the handler answered with something else than JSON\n{out}");
        };
        for message in &messages {
            assert!(
                message["error"]["code"].as_i64() != Some(INTERNAL_ERROR),
                "wasm_session: internal error {message}"
            );
        }
        messages
    }

    fn send(&mut self, message: &Value) -> Vec<Value> {
        self.handle(&message.to_string())
    }

    fn request(&mut self, method: &str, params: Value) -> Vec<Value> {
        self.send(&json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}))
    }

    fn configure(&mut self, settings: &Value) {
        self.send(&json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": {"settings": settings},
        }));
    }

    /// Opens the document, and returns the diagnostics it published.
    fn open(&mut self, text: &Text, version: i32) -> Value {
        let out = self.send(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {"textDocument": {"uri": URI, "languageId": "sql", "version": version, "text": text.0}},
        }));
        Self::diagnostics(&out)
    }

    /// Replaces the text of the document, and returns the diagnostics it published.
    fn change(&mut self, text: &Text, version: i32) -> Value {
        let out = self.send(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": URI, "version": version},
                "contentChanges": [{"text": text.0}],
            },
        }));
        Self::diagnostics(&out)
    }

    fn diagnostics(out: &[Value]) -> Value {
        let Some(published) = out
            .iter()
            .find(|message| message["method"] == "textDocument/publishDiagnostics")
        else {
            panic!("wasm_session: no diagnostics published\n{out:?}");
        };
        published["params"]["diagnostics"].clone()
    }
}

/// The text of the document, as the client sees it.
#[derive(Clone)]
struct Text(String);

impl Text {
    /// A char boundary, from an arbitrary number.
    fn byte_at(&self, at: u16) -> usize {
        let at = usize::from(at) % (self.0.len() + 1);
        (0..=at)
            .rev()
            .find(|&at| self.0.is_char_boundary(at))
            .unwrap_or(0)
    }

    /// The lines as LSP splits them, at `\r\n`, `\n` and `\r`, without the line breaks.
    fn lines(&self) -> Vec<&str> {
        let mut lines = Vec::new();
        let mut rest = self.0.as_str();
        while let Some(end) = rest.find(['\r', '\n']) {
            lines.push(&rest[..end]);
            let line_break = if rest[end..].starts_with("\r\n") { 2 } else { 1 };
            rest = &rest[end + line_break..];
        }
        lines.push(rest);
        lines
    }

    /// The LSP position of a byte offset, in UTF-16 code units. An offset between `\r` and `\n`
    /// is at the end of its line.
    fn position(&self, offset: usize) -> Value {
        let before = Text(self.0[..offset].to_owned());
        let lines = before.lines();
        let last = lines.last().copied().unwrap_or_default();
        let character: usize = last.chars().map(char::len_utf16).sum();
        json!({"line": lines.len() - 1, "character": character})
    }

    /// Every diagnostic lies inside the text.
    fn check(&self, diagnostics: &Value) {
        let lines = self.lines();
        for diagnostic in diagnostics.as_array().into_iter().flatten() {
            for end in ["start", "end"] {
                let position = &diagnostic["range"][end];
                let line = position["line"].as_u64().unwrap_or(u64::MAX);
                let character = position["character"].as_u64().unwrap_or(u64::MAX);
                let width = usize::try_from(line)
                    .ok()
                    .and_then(|line| lines.get(line))
                    .map(|line| line.chars().map(char::len_utf16).sum::<usize>());
                assert!(
                    width.is_some_and(|width| character <= width as u64),
                    "wasm_session: a diagnostic lies outside the text\n{diagnostic}\ntext: {:?}",
                    self.0
                );
            }
        }
    }
}
