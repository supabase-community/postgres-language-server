The goal is to port all missing rules from Squawk to our analyser.

Our analyser lives in the `pgt_analyser` crate. There is a `CONTRIBUTING.md` guide in that crate which explains how to add new rules. Please also read existing rules to see how it all works.

Then, I want you to check the rules in the squawk project which I copied here for convenience. The rules are in `squawk/linter/src/rules`. The implementation should be very similar to what we have, and porting them straightforward. Here a few things to watch out for though:

- although both libraries are using `libpg_query` to parse the SQL, the bindings can be different. Ours is in the `pgt_query` crate of you need a reference. The `protobuf.rs` file contains the full thing.
- the context for each rule is different, but you can get the same information out of it:
```rust
pub struct RuleContext<'a, R: Rule> {
    // the ast of the target statement
    stmt: &'a pgt_query::NodeEnum,
    // options for that specific rule
    options: &'a R::Options,
    // the schema cache - also includes the postgres version
    schema_cache: Option<&'a SchemaCache>,
    // the file context which contains other statements in that file in case you need them
    file_context: &'a AnalysedFileContext,
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

If you learn something new that might help in porting all the rules, please update this document.

LEARNINGS:


Please update the list below with the rules that we need to migrate, and the ones that are already migrated. Keep the list up-to-date.

TODO:
- adding_field_with_default
- adding_foreign_key_constraint
- adding_not_null_field
- adding_primary_key_constraint
- ban_char_field
- ban_concurrent_index_creation_in_transaction
- changing_column_type
- constraint_missing_not_valid
- disallow_unique_constraint
- prefer_big_int
- prefer_bigint_over_int
- prefer_bigint_over_smallint
- prefer_identity
- prefer_robust_stmts
- prefer_text_field
- prefer_timestamptz
- renaming_column
- renaming_table
- require_concurrent_index_creation
- require_concurrent_index_deletion
- transaction_nesting

DONE:
- adding_required_field (already exists in pgt_analyser)
- ban_drop_column (already exists in pgt_analyser)
- ban_drop_database (already exists in pgt_analyser, as bad_drop_database in squawk)
- ban_drop_not_null (already exists in pgt_analyser)
- ban_drop_table (already exists in pgt_analyser)


