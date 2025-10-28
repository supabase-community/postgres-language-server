# Porting Rules from Eugene to postgres-language-server

This document tracks the progress of porting lint rules from [Eugene](https://github.com/kaaveland/eugene) to the postgres-language-server analyzer.

## Overview

Eugene is a PostgreSQL migration linter that detects dangerous operations. We are porting its rules to the postgres-language-server to provide similar safety checks within the language server environment.

**Eugene source location**: `eugene/eugene/src/lints/rules.rs`
**Hint metadata location**: `eugene/eugene/src/hint_data.rs`
**Example SQL files**: `eugene/eugene/examples/*/`

## Step-by-Step Porting Process

### 1. Understand the Rule

1. **Read Eugene's implementation** in `eugene/eugene/src/lints/rules.rs`
   - Find the rule function (e.g., `added_serial_column`)
   - Understand the AST patterns it matches
   - Note any special logic (e.g., checking previous statements)

2. **Read the hint metadata** in `eugene/eugene/src/hint_data.rs`
   - Get the ID (e.g., "E11")
   - Get name, condition, effect, and workaround text
   - This provides documentation content

3. **Review example SQL** in `eugene/eugene/examples/<ID>/`
   - `bad.sql` - Invalid cases that should trigger the rule
   - `good.sql` - Valid cases that should NOT trigger

### 2. Create the Rule

```bash
# Create rule with appropriate severity (error/warn)
just new-lintrule safety <ruleName> <severity>

# Example:
just new-lintrule safety addSerialColumn error
```

This generates:
- `crates/pgls_analyser/src/lint/safety/<rule_name>.rs`
- `crates/pgls_analyser/tests/specs/safety/<ruleName>/basic.sql`
- Updates to configuration files
- Diagnostic category registration

### 3. Implement the Rule

**File**: `crates/pgls_analyser/src/lint/safety/<rule_name>.rs`

#### Key Components:

```rust
use pgls_analyse::{context::RuleContext, declare_lint_rule, Rule, RuleDiagnostic, RuleSource};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Brief one-line description (shown in lists).
    ///
    /// Detailed explanation of what the rule detects and why it's problematic.
    /// Explain the PostgreSQL behavior and performance/safety implications.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// -- SQL that should trigger the rule
    /// ALTER TABLE users ADD COLUMN id serial;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// -- SQL that should NOT trigger
    /// CREATE TABLE users (id serial PRIMARY KEY);
    /// ```
    ///
    pub RuleName {
        version: "next",
        name: "ruleName",
        severity: Severity::Error,  // or Warning
        recommended: true,  // or false
        sources: &[RuleSource::Eugene("<ID>")],  // e.g., "E11"
    }
}

impl Rule for RuleName {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // Pattern match on the statement type
        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            // Rule logic here

            if /* condition */ {
                diagnostics.push(
                    RuleDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Error message with "<Emphasis>"formatting"</Emphasis>"."
                        },
                    )
                    .detail(None, "Additional context about the problem.")
                    .note("Suggested fix or workaround."),
                );
            }
        }

        diagnostics
    }
}
```

#### Important Patterns:

**Accessing previous statements** (for rules like `multipleAlterTable`):
```rust
let file_ctx = ctx.file_context();
let previous = file_ctx.previous_stmts();
```

**Schema normalization** (treating empty schema as "public"):
```rust
let schema_normalized = if schema.is_empty() {
    "public"
} else {
    schema.as_str()
};
```

**Checking for specific ALTER TABLE actions**:
```rust
for cmd in &stmt.cmds {
    if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
        if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn {
            // Handle ADD COLUMN
        }
    }
}
```

**Extracting column type**:
```rust
if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = &cmd.def.as_ref().and_then(|d| d.node.as_ref()) {
    if let Some(type_name) = &col_def.type_name {
        let type_str = get_type_name(type_name);
    }
}

fn get_type_name(type_name: &pgls_query::protobuf::TypeName) -> String {
    type_name
        .names
        .iter()
        .filter_map(|n| {
            if let Some(pgls_query::NodeEnum::String(s)) = &n.node {
                Some(s.sval.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}
```

### 4. Create Comprehensive Tests

**Directory**: `crates/pgls_analyser/tests/specs/safety/<ruleName>/`

Create multiple test files covering:

#### Invalid Cases (should trigger):
```sql
-- expect_lint/safety/<ruleName>
-- Description of what this tests
<SQL that should trigger>
```

#### Valid Cases (should NOT trigger):
```sql
-- Description of what this tests
-- expect_no_diagnostics
<SQL that should NOT trigger>
```

#### Example Test Structure:
```
tests/specs/safety/addSerialColumn/
├── basic.sql                    # Basic case triggering the rule
├── bigserial.sql               # Variant (bigserial type)
├── generated_stored.sql        # Another variant (GENERATED)
├── valid_regular_column.sql    # Valid: regular column
└── valid_create_table.sql      # Valid: CREATE TABLE context
```

**Run tests and accept snapshots**:
```bash
cargo insta test -p pgls_analyser --accept
```

### 5. Verify and Generate Code

```bash
# Check compilation
cargo check

# Generate lint code and documentation
just gen-lint

# Run all tests
cargo test -p pgls_analyser --test rules_tests

# Final verification
just ready
```

### 6. Test the Rule Manually

Create a test SQL file:
```sql
-- test.sql
ALTER TABLE users ADD COLUMN id serial;
```

Run the CLI:
```bash
cargo run -p pgls_cli -- check /path/to/test.sql
```

## Common Pitfalls and Solutions

### 1. Schema Matching

**Problem**: Need to match tables across statements with different schema notations.

**Solution**: Normalize schema names:
```rust
let schema_normalized = if schema.is_empty() { "public" } else { schema.as_str() };
```

### 2. AST Navigation

**Problem**: Eugene uses a simplified AST (`StatementSummary`), but pgt uses the full PostgreSQL AST.

**Solution**: Use pattern matching and helper functions. Look at existing rules for examples.

### 3. File Context Rules

**Problem**: Rules that need to track state across statements (like `multipleAlterTable`).

**Solution**: Use `ctx.file_context()` to access `AnalysedFileContext`:
```rust
let file_ctx = ctx.file_context();
let previous_stmts = file_ctx.previous_stmts();
```

### 4. Transaction State

**Problem**: Some Eugene rules check transaction state (e.g., `RUNNING_STATEMENT_WHILE_HOLDING_ACCESS_EXCLUSIVE`).

**Solution**: This is more complex and may require extending the `AnalysedFileContext` to track transaction state. Consider implementing simpler rules first.

### 5. Test Expectations

**Problem**: `expect_lint` expects exactly ONE diagnostic, but rule generates multiple.

**Solution**: Either:
- Split into separate test files (one diagnostic each)
- Adjust the test to only trigger once
- Use expect_diagnostic for each occurrence (check existing tests)

## Rule Mapping Considerations

### Overlapping Rules

Some Eugene rules may overlap with existing pgt rules:

| Eugene Rule | Potential PGT Overlap | Action |
|-------------|----------------------|--------|
| `SET_COLUMN_TYPE_TO_JSON` | `preferJsonb` | Review both, may enhance existing |
| `CREATE_INDEX_NONCONCURRENTLY` | `requireConcurrentIndexCreation` | Review both |
| `CHANGE_COLUMN_TYPE` | `changingColumnType` | Review both |
| `ADD_NEW_UNIQUE_CONSTRAINT_WITHOUT_USING_INDEX` | `disallowUniqueConstraint` | Review both |

**When overlap exists**:
1. Compare implementations
2. If Eugene's is more comprehensive, consider updating the existing rule
3. If they cover different aspects, keep both
4. Document any differences

### Transaction-Aware Rules

These rules require tracking transaction state across multiple statements:

- `RUNNING_STATEMENT_WHILE_HOLDING_ACCESS_EXCLUSIVE` (E4)
- `LOCKTIMEOUT_WARNING` (E9)

**Approach**:
1. First implement simpler, statement-level rules
2. Design transaction state tracking in `AnalysedFileContext`
3. Add fields to track:
   - Current transaction state (BEGIN/COMMIT/ROLLBACK)
   - Locks held
   - Lock timeout settings
4. Update state as statements are processed

## Eugene AST vs PostgreSQL AST

### Eugene's Simplified AST

Eugene uses `StatementSummary` enum with simplified representations:
```rust
enum StatementSummary {
    AlterTable { schema: String, name: String, actions: Vec<AlterTableAction> },
    CreateIndex { schema: String, idxname: String, concurrently: bool, target: String },
    // ...
}

enum AlterTableAction {
    AddColumn { column: String, type_name: String, stored_generated: bool, ... },
    SetType { column: String, type_name: String },
    // ...
}
```

### PostgreSQL AST (pgls_query)

We use the full PostgreSQL protobuf AST:
```rust
pgls_query::NodeEnum::AlterTableStmt(stmt)
  -> stmt.cmds: Vec<Node>
    -> NodeEnum::AlterTableCmd(cmd)
      -> cmd.subtype: AlterTableType
      -> cmd.def: Option<Node>
        -> NodeEnum::ColumnDef(col_def)
```

**Translation Strategy**:
1. Look at Eugene's simplified logic
2. Map to corresponding PostgreSQL AST nodes
3. Use existing pgt rules as references
4. Check `pgls_query::protobuf` for available types/enums

## Useful References

- **Eugene source**: `eugene/eugene/src/lints/rules.rs`
- **Existing pgt rules**: `crates/pgls_analyser/src/lint/safety/`
- **Contributing guide**: `crates/pgls_analyser/CONTRIBUTING.md`
- **AST types**: `crates/pgls_query/src/lib.rs`
- **PostgreSQL protobuf**: `pgls_query::protobuf` module

## Next Steps

1. **Priority**: Port high-priority safety rules (E1, E2, E6, E7, E9)
2. **Review overlaps**: Check if E3, E5, E6, E7 overlap with existing rules
3. **Transaction tracking**: Design transaction state tracking for E4, E9
4. **Documentation**: Update Eugene source attribution in ported rules
5. **Testing**: Ensure comprehensive test coverage for all ported rules

## Template Checklist

When porting a new rule, ensure:

- [ ] Rule implementation in `src/lint/safety/<rule>.rs`
- [ ] Documentation with examples (invalid and valid cases)
- [ ] `sources: &[RuleSource::Eugene("<ID>")]` attribution
- [ ] At least 3-5 test files (mix of invalid and valid)
- [ ] Snapshot tests accepted with `cargo insta test --accept`
- [ ] All tests pass: `cargo test -p pgls_analyser --test rules_tests`
- [ ] Compilation clean: `cargo check`
- [ ] Code generation: `just gen-lint`
- [ ] Manual CLI test with sample SQL
- [ ] Update this document with completed rule
