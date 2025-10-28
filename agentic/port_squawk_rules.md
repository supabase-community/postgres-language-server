The goal is to port all missing rules from Squawk to our analyser.

Our analyser lives in the `pgls_analyser` crate. There is a `CONTRIBUTING.md` guide in that crate which explains how to add new rules. Please also read existing rules to see how it all works.

Then, I want you to check the rules in the squawk project which I copied here for convenience. The rules are in `squawk/linter/src/rules`. The implementation should be very similar to what we have, and porting them straightforward. Here a few things to watch out for though:

- although both libraries are using `libpg_query` to parse the SQL, the bindings can be different. Ours is in the `pgls_query` crate of you need a reference. The `protobuf.rs` file contains the full thing.
- the context for each rule is different, but you can get the same information out of it:
```rust
pub struct RuleContext<'a, R: Rule> {
    // the ast of the target statement
    stmt: &'a pgls_query::NodeEnum,
    // options for that specific rule
    options: &'a R::Options,
    // the schema cache - also includes the postgres version
    schema_cache: Option<&'a SchemaCache>,
    // the file context which contains other statements in that file in case you need them
    file_context: &'a AnalysedFileContext,
}


pub struct AnalysedFileContext<'a> {
    // all statements in this file
    pub stmts: &'a Vec<pgls_query::NodeEnum>,

    pos: usize,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a Vec<pgls_query::NodeEnum>) -> Self {
        Self { stmts, pos: 0 }
    }

    // all statements before the currently analysed one
    pub fn previous_stmts(&self) -> &[pgls_query::NodeEnum] {
        &self.stmts[0..self.pos]
    }

    // total count of statements in this file
    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }
}
```

In squawk, you will see:
```rust
    // all statements of that file -> our analyser goes statement by statement but has access to the files content via `file_context`
    tree: &[RawStmt],
    // the postgres version -> we store it in the schema cache
    _pg_version: Option<Version>,
    // for us, this is always true
    _assume_in_transaction: bool,

```

Please always write idiomatic code!
Only add comments to explain WHY the code is doing something. DO NOT write comments to explain WHAT the code is doing.

If you learn something new that might help in porting all the rules, please update this document.

LEARNINGS:
- Use `cargo clippy` to check your code after writing it
- The `just new-lintrule` command expects severity to be "info", "warn", or "error" (not "warning")
- RuleDiagnostic methods: `detail(span, msg)` takes two parameters, `note(msg)` takes only one parameter
- To check Postgres version: access `ctx.schema_cache().is_some_and(|sc| sc.version.major_version)` which gives e.g. 17
- NEVER skip anything, or use a subset of something. ALWAYS do the full thing. For example, copy the entire non-volatile functions list from Squawk, not just a subset.
- If you are missing features from our rule context to be able to properly implement a rule, DO NOT DO IT. Instead, add that rule to the NEEDS FEATURES list below. The node enum is generated from the same source as it is in squawk, so they have feature parity.
- Remember to run `just gen-lint` after creating a new rule to generate all necessary files

Please update the list below with the rules that we need to migrate, and the ones that are already migrated. Keep the list up-to-date.

NEEDS FEATURES:

TODO:

DONE:
- adding_field_with_default ✓ (ported from Squawk)
- adding_foreign_key_constraint ✓ (ported from Squawk)
- adding_not_null_field ✓ (ported from Squawk)
- adding_primary_key_constraint ✓ (ported from Squawk)
- adding_required_field (already exists in pgls_analyser)
- ban_char_field ✓ (ported from Squawk)
- ban_concurrent_index_creation_in_transaction ✓ (ported from Squawk)
- ban_drop_column (already exists in pgls_analyser)
- changing_column_type ✓ (ported from Squawk)
- constraint_missing_not_valid ✓ (ported from Squawk)
- ban_drop_database (already exists in pgls_analyser, as bad_drop_database in squawk)
- ban_drop_not_null (already exists in pgls_analyser)
- ban_drop_table (already exists in pgls_analyser)
- prefer_big_int ✓ (ported from Squawk)
- prefer_bigint_over_int ✓ (ported from Squawk)
- prefer_bigint_over_smallint ✓ (ported from Squawk)
- prefer_identity ✓ (ported from Squawk)
- prefer_jsonb ✓ (new rule added)
- prefer_text_field ✓ (ported from Squawk)
- prefer_timestamptz ✓ (ported from Squawk)
- disallow_unique_constraint ✓ (ported from Squawk)
- renaming_column ✓ (ported from Squawk)
- renaming_table ✓ (ported from Squawk)
- require_concurrent_index_creation ✓ (ported from Squawk)
- require_concurrent_index_deletion ✓ (ported from Squawk)
- transaction_nesting ✓ (ported from Squawk)
- prefer_robust_stmts ✓ (ported from Squawk - simplified version)


