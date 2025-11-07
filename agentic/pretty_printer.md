# Pretty Printer Implementation Plan

## Overview

This document outlines the plan to complete the implementation of the Postgres SQL pretty printer in `crates/pgt_pretty_print/`. The pretty printer takes parsed SQL AST nodes (from `pgt_query`) and emits formatted SQL code that respects line length constraints while maintaining semantic correctness.

## ⚠️ SCOPE: Implementation Task

**THIS TASK IS ONLY ABOUT IMPLEMENTING `emit_*` FUNCTIONS IN `src/nodes/`**

- ✅ **DO**: Implement `emit_*` functions for each AST node type
- ✅ **DO**: Add new files to `src/nodes/` for each node type
- ✅ **DO**: Update `src/nodes/mod.rs` to dispatch new node types
- ✅ **DO**: Use existing helpers in `node_list.rs` and `string.rs`
- ✅ **DO**: Keep this document updated with progress and learnings
- ❌ **DON'T**: Modify the renderer (`src/renderer.rs`)
- ❌ **DON'T**: Modify the emitter (`src/emitter.rs`)
- ❌ **DON'T**: Change the test infrastructure (`tests/tests.rs`)
- ❌ **DON'T**: Modify code generation (`src/codegen/`)

The renderer, emitter, and test infrastructure are already complete and working correctly. Your job is to implement the missing `emit_*` functions so that all AST nodes can be formatted.

## 📝 CRITICAL: Keep This Document Updated

**As you implement nodes, update the following sections:**

1. **Completed Nodes section** - Mark nodes as `[x]` when done, add notes about partial implementations
2. **Implementation Learnings section** - Add or prune concise bullets capturing durable guidance (no long session logs)
3. **Progress tracking** - Update the count (e.g., "14/270 → 20/270") or note it in the "Next Steps" section

**This allows stopping and restarting work at any time!**

## Architecture

### Core Components

1. **EventEmitter** (`src/emitter.rs`)
   - Emits layout events (tokens, spaces, lines, groups, indents)
   - Events are later processed by the renderer to produce formatted output

2. **Renderer** (`src/renderer.rs`)
   - Converts layout events into actual formatted text
   - Handles line breaking decisions based on `max_line_length`
   - Implements group-based layout algorithm

3. **Node Emission** (`src/nodes/`)
   - One file per AST node type (e.g., `select_stmt.rs`, `a_expr.rs`)
   - Each file exports an `emit_*` function that takes `&mut EventEmitter` and the node

4. **Code Generation** (`src/codegen/`)
   - `TokenKind`: Generated enum for all SQL tokens (keywords, operators, punctuation)
   - `GroupKind`: Generated enum for logical groupings of nodes

## Implementation Pattern

### Standard Node Emission Pattern

Each `emit_*` function follows this pattern:

```rust
pub(super) fn emit_<node_name>(e: &mut EventEmitter, n: &<NodeType>) {
    // 1. Start a group for this node
    e.group_start(GroupKind::<NodeName>);

    // 2. Emit keywords
    e.token(TokenKind::KEYWORD_KW);

    // 3. Emit child nodes with spacing/line breaks
    if let Some(ref child) = n.child {
        e.space(); // or e.line(LineType::SoftOrSpace)
        super::emit_node(child, e);
    }

    // 4. Emit lists with separators
    emit_comma_separated_list(e, &n.items, super::emit_node);

    // 5. End the group
    e.group_end();
}
```

### Pattern Variations and Examples

#### 1. Simple Node with Fields (RangeVar)

When a node has simple string fields and no optional complex children:

```rust
// src/nodes/range_var.rs
pub(super) fn emit_range_var(e: &mut EventEmitter, n: &RangeVar) {
    e.group_start(GroupKind::RangeVar);

    // Emit qualified name: schema.table
    if !n.schemaname.is_empty() {
        super::emit_identifier_maybe_quoted(e, &n.schemaname);
        e.token(TokenKind::DOT);
    }

    super::emit_identifier_maybe_quoted(e, &n.relname);

    e.group_end();
}
```

**Key points**:
- No spaces around DOT token
- Check if optional fields are empty before emitting
- Reuse the helpers in `string.rs` (`emit_identifier_maybe_quoted`, etc.) instead of hand-emitting `TokenKind::IDENT`

#### 2. Node with List Helper (ColumnRef)

When a node primarily wraps a list:

```rust
// src/nodes/column_ref.rs
pub(super) fn emit_column_ref(e: &mut EventEmitter, n: &ColumnRef) {
    e.group_start(GroupKind::ColumnRef);
    emit_dot_separated_list(e, &n.fields);
    e.group_end();
}
```

**Key points**:
- Delegate to helper functions in `node_list.rs`
- Available helpers:
  - `emit_comma_separated_list(e, nodes, render_fn)`
  - `emit_dot_separated_list(e, nodes)`
  - `emit_keyword_separated_list(e, nodes, keyword)`

#### 3. Context-Specific Emission (ResTarget)

When a node needs different formatting based on context (SELECT vs UPDATE):

```rust
// src/nodes/res_target.rs

// For SELECT target list: "expr AS alias"
pub(super) fn emit_res_target(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);

    if let Some(ref val) = n.val {
        emit_node(val, e);
    } else {
        return;
    }

    emit_column_name_with_indirection(e, n);

    if !n.name.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        emit_identifier(e, &n.name);
    }

    e.group_end();
}

// For UPDATE SET clause: "column = expr"
pub(super) fn emit_set_clause(e: &mut EventEmitter, n: &ResTarget) {
    e.group_start(GroupKind::ResTarget);

    if n.name.is_empty() {
        return;
    }

    emit_column_name_with_indirection(e, n);

    if let Some(ref val) = n.val {
        e.space();
        e.token(TokenKind::IDENT("=".to_string()));
        e.space();
        emit_node(val, e);
    }

    e.group_end();
}

// Shared helper for column name with array/field access
pub(super) fn emit_column_name_with_indirection(e: &mut EventEmitter, n: &ResTarget) {
    if n.name.is_empty() {
        return;
    }

    e.token(TokenKind::IDENT(n.name.clone()));

    for i in &n.indirection {
        match &i.node {
            // Field selection: column.field
            Some(pgt_query::NodeEnum::String(n)) => super::emit_string_identifier(e, n),
            // Other indirection types (array access, etc.)
            Some(n) => super::emit_node_enum(n, e),
            None => {}
        }
    }
}
```

**Key points**:
- Export multiple `pub(super)` functions for different contexts
- Share common logic in helper functions
- Handle indirection (array access, field selection) carefully

#### 4. Using `assert_node_variant!` Macro (UpdateStmt)

When you need to extract a specific node variant from a generic `Node`:

```rust
// src/nodes/update_stmt.rs
use crate::nodes::res_target::emit_set_clause;

pub(super) fn emit_update_stmt(e: &mut EventEmitter, n: &UpdateStmt) {
    e.group_start(GroupKind::UpdateStmt);

    e.token(TokenKind::UPDATE_KW);
    e.space();

    if let Some(ref range_var) = n.relation {
        super::emit_range_var(e, range_var)
    }

    if !n.target_list.is_empty() {
        e.space();
        e.token(TokenKind::SET_KW);
        e.space();

        // Use assert_node_variant! to extract ResTarget from generic Node
        emit_comma_separated_list(e, &n.target_list, |n, e| {
            emit_set_clause(e, assert_node_variant!(ResTarget, n))
        });
    }

    if let Some(ref where_clause) = n.where_clause {
        e.space();
        e.token(TokenKind::WHERE_KW);
        e.space();
        emit_node(where_clause, e);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
```

**Key points**:
- `assert_node_variant!(NodeType, expr)` extracts a specific node type
- Use this when you know the list contains a specific node type
- Panics if the variant doesn't match (design-time check)
- Useful in closures passed to list helpers

### Important Macros and Helpers

#### String Emission Helpers (`src/nodes/string.rs`)

The `string.rs` module provides helpers for emitting SQL identifiers, literals, and keywords with proper quoting:

**Available Functions**:

```rust
// Emit with smart quoting (quotes only if needed: keywords, uppercase, special chars)
emit_identifier_maybe_quoted(e, "users")     // → users
emit_identifier_maybe_quoted(e, "User")      // → "User"
emit_identifier_maybe_quoted(e, "select")    // → "select"

// Always emit with double quotes (for case-sensitive identifiers)
emit_identifier(e, "MyTable")                // → "MyTable"
emit_identifier(e, "en_US")                  // → "en_US"

// Emit single-quoted string literals
emit_single_quoted_str(e, "hello")           // → 'hello'
emit_single_quoted_str(e, "it's")            // → 'it''s'

// Emit dollar-quoted string literals (for function bodies, DO blocks)
emit_dollar_quoted_str(e, "SELECT 1")        // → $$SELECT 1$$
emit_dollar_quoted_str(e, "has $$")          // → $pg$has $$$pg$

// Emit SQL keywords (converts to TokenKind if available)
emit_keyword(e, "LANGUAGE")                  // → TokenKind::LANGUAGE_KW
```

**For String nodes from AST**:

```rust
use pgt_query::protobuf::String as PgString;

// Smart quoting for identifiers (column names, table names)
emit_string(e, &string_node)                 // → calls emit_identifier_maybe_quoted

// Always quote (collation names, case-sensitive contexts)
emit_string_identifier(e, &string_node)      // → calls emit_identifier

// String literal (passwords, file paths, enum values)
emit_string_literal(e, &string_node)         // → calls emit_single_quoted_str
```

**Usage Guidelines**:
- **Default choice**: Use `emit_identifier_maybe_quoted()` for most identifiers (column/table names)
- **Force quotes**: Use `emit_identifier()` when case must be preserved (collations, mixed-case names)
- **String literals**: Use `emit_single_quoted_str()` for SQL string values
- **Large text blocks**: Use `emit_dollar_quoted_str()` for function bodies, DO blocks
- **Keywords**: Use `emit_keyword()` to automatically get the right TokenKind

#### `assert_node_variant!` Macro

Defined in `src/nodes/mod.rs`:

```rust
macro_rules! assert_node_variant {
    ($variant:ident, $expr:expr) => {
        match $expr.node.as_ref() {
            Some(pgt_query::NodeEnum::$variant(inner)) => inner,
            other => panic!("Expected {}, got {:?}", stringify!($variant), other),
        }
    };
}
```

**Usage**:
```rust
// When you have a Node and need a specific type
let res_target = assert_node_variant!(ResTarget, node);
emit_res_target(e, res_target);

// In closures for list helpers
emit_comma_separated_list(e, &n.target_list, |node, e| {
    let res_target = assert_node_variant!(ResTarget, node);
    emit_res_target(e, res_target);
});
```

**When to use**:
- When iterating over a `Vec<Node>` that you know contains specific types
- The macro panics at runtime if the type doesn't match (indicates a bug)
- This is better than unwrapping because it provides a clear error message

**Best Practices for AST Assertions**:

1. **Use `assert_node_variant!` instead of `if let Some(NodeEnum::...)`** when you expect a specific type:
   ```rust
   // ❌ Weak: silently skips unexpected types
   if let Some(NodeEnum::DefElem(def_elem)) = &arg.node {
       // handle def_elem
   }

   // ✅ Strong: fails fast with clear error
   let def_elem = assert_node_variant!(DefElem, arg);
   // handle def_elem
   ```

2. **Use `debug_assert!` for structural expectations**:
   ```rust
   debug_assert!(
       n.args.len() == 2,
       "ScalarArrayOpExpr should have exactly 2 args, got {}",
       n.args.len()
   );
   ```

3. **Use `if let` for genuinely optional variants**:
   ```rust
   // When a node might be multiple types and you handle each differently
   match &node.node {
       Some(NodeEnum::AArrayExpr(arr)) => emit_as_array(e, arr),
       Some(NodeEnum::SubLink(sub)) => emit_as_subquery(e, sub),
       other => emit_generic(e, other),
   }
   ```

4. **For `DefElem`-driven nodes, extract all fields first, then validate**:
   ```rust
   let mut language: Option<String> = None;
   let mut body: Option<String> = None;

   for arg in &n.args {
       let def_elem = assert_node_variant!(DefElem, arg);
       match def_elem.defname.as_str() {
           "language" => {
               let s = assert_node_variant!(String, def_elem.arg.as_ref().unwrap());
               language = Some(s.sval.clone());
           }
           "as" => { /* ... */ }
           other => debug_assert!(false, "Unexpected defname '{}'", other),
       }
   }

   debug_assert!(language.is_some(), "Missing required 'language' field");
   debug_assert!(body.is_some(), "Missing required 'as' field");
   ```

#### Node Dispatch Pattern

The main dispatch in `src/nodes/mod.rs`:

```rust
pub fn emit_node(node: &Node, e: &mut EventEmitter) {
    if let Some(ref inner) = node.node {
        emit_node_enum(inner, e)
    }
}

pub fn emit_node_enum(node: &NodeEnum, e: &mut EventEmitter) {
    match &node {
        NodeEnum::SelectStmt(n) => emit_select_stmt(e, n),
        NodeEnum::UpdateStmt(n) => emit_update_stmt(e, n),
        // ... more cases
        _ => todo!("emit_node_enum: unhandled node type {:?}", node),
    }
}
```

**To add a new node**:
1. Create `src/nodes/<node_name>.rs`
2. Add `mod <node_name>;` to `src/nodes/mod.rs`
3. Add `use <node_name>::emit_<node_name>;` to imports
4. Add case to `emit_node_enum` match

### Layout Event Types

- **Token**: An actual SQL keyword/operator/identifier (e.g., `SELECT`, `+`, `,`)
- **Space**: A single space character
- **Line**: A line break with different behaviors:
  - `Hard`: Always breaks (e.g., after semicolon)
  - `Soft`: Breaks if group doesn't fit
  - `SoftOrSpace`: Becomes a space if group fits, line break otherwise
- **GroupStart/GroupEnd**: Logical grouping for layout decisions
- **IndentStart/IndentEnd**: Increase/decrease indentation level

### Inspirations from Go Parser

The Go parser in `parser/ast/*.go` provides reference implementations via `SqlString()` methods:

1. **Statement Files**:
   - `statements.go`: SELECT, INSERT, UPDATE, DELETE, CREATE, DROP
   - `ddl_statements.go`: CREATE TABLE, ALTER TABLE, etc.
   - `administrative_statements.go`: GRANT, REVOKE, etc.
   - `utility_statements.go`: COPY, VACUUM, etc.

2. **Expression Files**:
   - `expressions.go`: A_Expr, BoolExpr, ColumnRef, FuncCall, etc.
   - `type_coercion_nodes.go`: TypeCast, CollateClause, etc.

3. **Key Methods to Reference**:
   - `SqlString()`: Returns the SQL string representation
   - `FormatFullyQualifiedName()`: Handles schema.table.column formatting
   - `QuoteIdentifier()`: Adds quotes when needed
   - `FormatCommaList()`: Comma-separated lists

### Inspiration from pgFormatter

Use `pgFormatter` to get ideas about line breaking and formatting decisions:

```bash
# Format a test file to see how pgFormatter would handle it
pg_format tests/data/single/your_test_80.sql

# Format with specific line width
pg_format -w 60 tests/data/single/your_test_60.sql

# Format and output to file for comparison
pg_format tests/data/single/complex_query_80.sql > /tmp/formatted.sql
```

**When to use pgFormatter for inspiration**:
- **Line breaking decisions**: Where should clauses break?
- **Indentation levels**: How much to indent nested structures?
- **Spacing conventions**: Spaces around operators, keywords, etc.
- **Complex statements**: JOINs, CTEs, window functions, etc.

**Important notes**:
- pgFormatter output is for **inspiration only** - don't copy exactly
- Our pretty printer uses a **group-based algorithm** (different from pgFormatter)
- Focus on using **groups and line types** (Soft, SoftOrSpace, Hard) rather than trying to replicate exact output
- pgFormatter might make different choices - that's OK! Use it as a reference, not a spec

**Example workflow**:
```bash
# 1. Create your test case
echo "SELECT a, b, c FROM table1 JOIN table2 ON table1.id = table2.id WHERE x > 10" > tests/data/single/join_example_80.sql

# 2. See how pgFormatter would format it
pg_format -w 80 tests/data/single/join_example_80.sql

# 3. Use that as inspiration for your emit_* implementation
# 4. Run your test to see your output
cargo test -p pgt_pretty_print test_single__join_example_80 -- --show-output

# 5. Iterate on your implementation
```

### Mapping Go to Rust

| Go Pattern | Rust Pattern |
|------------|--------------|
| `parts = append(parts, "SELECT")` | `e.token(TokenKind::SELECT_KW)` |
| `strings.Join(parts, " ")` | Sequential `e.space()` calls |
| `strings.Join(items, ", ")` | `emit_comma_separated_list(...)` |
| `fmt.Sprintf("(%s)", expr)` | `e.token(LPAREN)`, emit, `e.token(RPAREN)` |
| String concatenation | Layout events (token + space/line) |
| `if condition { append(...) }` | `if condition { e.token(...) }` |

## Test Suite

### Test Structure

Tests are located in `tests/`:

1. **Single Statement Tests** (`tests/data/single/*.sql`)
   - Format: `<description>_<line_length>.sql`
   - Example: `simple_select_80.sql` → max line length of 80
   - Each test contains a single SQL statement

2. **Multi Statement Tests** (`tests/data/multi/*.sql`)
   - Format: `<description>_<line_length>.sql`
   - Contains multiple SQL statements separated by semicolons

### Running Tests

```bash
# Run all pretty print tests
cargo test -p pgt_pretty_print

# Run tests and update snapshots
cargo insta review

# Run a specific test
cargo test -p pgt_pretty_print test_single
```

### Test Validation

Each test validates:

1. **Line Length**: No line exceeds `max_line_length` (except for string literals)
2. **AST Equality**: Parsing the formatted output produces the same AST as the original
3. **Snapshot Match**: Output matches the stored snapshot

### Adding New Tests

You can and should create new test cases to validate your implementations!

1. **Create test file**:
   ```bash
   # For single statement tests
   echo "SELECT * FROM users WHERE age > 18" > tests/data/single/user_query_80.sql

   # For multi-statement tests
   cat > tests/data/multi/example_queries_60.sql <<'EOF'
   SELECT id FROM users;
   INSERT INTO logs (message) VALUES ('test');
   EOF
   ```

2. **Naming convention**: `<descriptive_name>_<line_length>.sql`
   - The number at the end is the max line length (e.g., `60`, `80`, `120`)
   - Examples: `complex_join_80.sql`, `insert_with_cte_60.sql`

3. **Run specific test**:
   ```bash
   # Run single test with output
   cargo test -p pgt_pretty_print test_single__user_query_80 -- --show-output

   # Run all tests matching pattern
   cargo test -p pgt_pretty_print test_single -- --show-output
   ```

4. **Review snapshots**:
   ```bash
   # Generate/update snapshots
   cargo insta review

   # Accept all new snapshots
   cargo insta accept
   ```

5. **Iterate**: Adjust your `emit_*` implementation based on test output

## Feedback Loop

### Development Workflow

1. **Identify a Node Type**
   - Look at test failures to see which node types are unimplemented
   - Check `src/nodes/mod.rs` for the `todo!()` in `emit_node_enum`

2. **Study the Go Implementation and pgFormatter**
   - Find the corresponding node in `parser/ast/*.go`
   - Study its `SqlString()` method for SQL structure
   - Use pgFormatter for line breaking ideas: `pg_format tests/data/single/your_test.sql`
   - Understand the structure and formatting rules

3. **Create Rust Implementation**
   - Create new file: `src/nodes/<node_name>.rs`
   - Implement `emit_<node_name>` function
   - Add to `mod.rs` imports and dispatch

4. **Test and Iterate**
   ```bash
   # Run tests to see if implementation works
   cargo test -p pgt_pretty_print

   # Review snapshots
   cargo insta review

   # Check specific test output
   cargo test -p pgt_pretty_print -- <test_name> --nocapture
   ```

5. **Refine Layout**
   - Adjust group boundaries for better breaking behavior
   - Use `SoftOrSpace` for clauses that can stay on one line
   - Use `Soft` for items that should prefer breaking
   - Add indentation for nested structures

### Debugging Tips

1. **Compare Snapshots**: Use `cargo insta review` to see diffs

2. **Check Parsed AST**: All tests print both old and new content as well as the old AST. If ASTs do not match, they show both. Run the tests with `-- --show-output` to see the stdout. This will help to see if an emit function misses a few properties of the node.

## Key Patterns and Best Practices

### 1. Group Boundaries

Groups determine where the renderer can break lines. Good practices:

- **Statement-level groups**: Wrap entire statements (SELECT, INSERT, etc.)
- **Clause-level groups**: Each clause (FROM, WHERE, ORDER BY) in a group
- **Expression-level groups**: Function calls, case expressions, parenthesized expressions

### 2. Line Break Strategy

- **After major keywords**: `SELECT`, `FROM`, `WHERE`, `ORDER BY`
  - Use `LineType::SoftOrSpace` to allow single-line for short queries
- **Between list items**: Comma-separated lists
  - Use `LineType::SoftOrSpace` after commas
- **Around operators**: Binary operators in expressions
  - Generally use spaces, not line breaks (handled by groups)

### 3. Indentation

- **Start indent**: After major keywords that introduce multi-item sections
  ```rust
  e.token(TokenKind::SELECT_KW);
  e.indent_start();
  e.line(LineType::SoftOrSpace);
  emit_comma_separated_list(e, &n.target_list, super::emit_node);
  e.indent_end();
  ```

- **Nested structures**: Subqueries, CASE expressions, function arguments

### 4. Whitespace Handling

- **Space before/after**: Most keywords and operators need spaces
- **No space**: Between qualifiers (`schema.table`, `table.column`)
- **Conditional space**: Use groups to let renderer decide

### 5. Special Cases

- **Parentheses**: Always emit as tokens, group contents
  ```rust
  e.token(TokenKind::LPAREN);
  e.group_start(GroupKind::ParenExpr);
  super::emit_node(&n.expr, e);
  e.group_end();
  e.token(TokenKind::RPAREN);
  ```

- **String literals**: Emit as tokens (no formatting inside)
- **Identifiers**: May need quoting (handled in token rendering)
- **Operators**: Can be keywords (`AND`) or symbols (`+`, `=`)

## Node Coverage Checklist

**Total Nodes**: ~270 node types from `pgt_query::protobuf::NodeEnum`

### Implementation Approach

**You can implement nodes partially!** For complex nodes with many fields:
1. Implement basic/common fields first
2. Add `todo!()` or comments for unimplemented parts
3. Test with simple cases
4. Iterate and add more fields as needed

Example partial implementation:
```rust
pub(super) fn emit_select_stmt(e: &mut EventEmitter, n: &SelectStmt) {
    e.group_start(GroupKind::SelectStmt);

    e.token(TokenKind::SELECT_KW);
    // Emit target list
    // TODO: DISTINCT clause
    // TODO: Window clause
    // TODO: GROUP BY
    // TODO: HAVING
    // TODO: ORDER BY
    // TODO: LIMIT/OFFSET

    e.group_end();
}
```

### Completed Nodes (248/270) - Last Updated 2025-11-06 Session 79
- [x] AArrayExpr (array literals ARRAY[...])
- [x] AConst (with all variants: Integer, Float, Boolean, String, BitString)
- [x] AExpr (partial - basic binary operators)
- [x] AIndices (array subscripts [idx] and slices [lower:upper])
- [x] AIndirection (array/field access operators)
- [x] AStar
- [x] AccessPriv (helper for GRANT/REVOKE privilege specifications)
- [x] Alias (AS aliasname with optional column list, fixed to not quote simple identifiers)
- [x] Aggref (planner aggregate nodes routed through the deparse bridge with guarded fallback)
- [x] AlterCollationStmt (ALTER COLLATION REFRESH VERSION)
- [x] AlterDatabaseStmt (ALTER DATABASE with options)
- [x] AlterDatabaseSetStmt (ALTER DATABASE SET configuration parameters)
- [x] AlterDatabaseRefreshCollStmt (ALTER DATABASE REFRESH COLLATION VERSION)
- [x] AlterDefaultPrivilegesStmt (ALTER DEFAULT PRIVILEGES)
- [x] AlterDomainStmt (ALTER DOMAIN with SET DEFAULT, DROP NOT NULL, ADD CONSTRAINT, etc.)
- [x] AlterEnumStmt (ALTER TYPE enum ADD VALUE, RENAME VALUE)
- [x] AlterEventTrigStmt (ALTER EVENT TRIGGER ENABLE/DISABLE)
- [x] AlterExtensionStmt (ALTER EXTENSION with UPDATE TO, ADD, DROP)
- [x] AlterExtensionContentsStmt (ALTER EXTENSION ADD/DROP object)
- [x] AlterFdwStmt (ALTER FOREIGN DATA WRAPPER)
- [x] AlterForeignServerStmt (ALTER SERVER with VERSION, OPTIONS)
- [x] AlterFunctionStmt (ALTER FUNCTION/PROCEDURE with function options)
- [x] AlterObjectDependsStmt (ALTER FUNCTION DEPENDS ON EXTENSION)
- [x] AlterObjectSchemaStmt (ALTER object SET SCHEMA)
- [x] AlterOperatorStmt (ALTER OPERATOR ... SET with commutator/negator/hash/merge options)
- [x] AlterOpFamilyStmt (ALTER OPERATOR FAMILY ADD/DROP)
- [x] AlterOwnerStmt (ALTER object_type name OWNER TO new_owner)
- [x] AlterPolicyStmt (ALTER POLICY with TO roles, USING, WITH CHECK)
- [x] AlterPublicationStmt (ALTER PUBLICATION ADD/DROP/SET)
- [x] AlterRoleStmt (ALTER ROLE with role options)
- [x] AlterRoleSetStmt (ALTER ROLE SET configuration IN DATABASE)
- [x] AlterSeqStmt (ALTER SEQUENCE with sequence options)
- [x] AlterStatsStmt (ALTER STATISTICS [IF EXISTS] SET STATISTICS)
- [x] AlterSubscriptionStmt (ALTER SUBSCRIPTION with 8 operation kinds)
- [x] AlterSystemStmt (ALTER SYSTEM wraps VariableSetStmt)
- [x] AlterTableStmt (ALTER TABLE with multiple subcommands: ADD COLUMN, DROP COLUMN, ALTER COLUMN, SET/DROP DEFAULT, ADD/DROP CONSTRAINT, etc.)
- [x] AlterTableCmd (standalone ALTER TABLE subcommands; shares formatting with AlterTableStmt dispatcher)
- [x] AlterTableMoveAllStmt (ALTER TABLE ALL IN TABLESPACE ... SET TABLESPACE ...)
- [x] AlterTableSpaceOptionsStmt (ALTER TABLESPACE with SET/RESET options)
- [x] AlterTsconfigurationStmt (ALTER TEXT SEARCH CONFIGURATION with ADD/ALTER/DROP MAPPING)
- [x] AlterTsdictionaryStmt (ALTER TEXT SEARCH DICTIONARY with options)
- [x] AlterTypeStmt (ALTER TYPE with option lists for OWNER/SET variants)
- [x] AlterUserMappingStmt (ALTER USER MAPPING FOR user SERVER server)
- [x] ArrayCoerceExpr (array coercions that simply forward the inner expression)
- [x] ArrayExpr (planner array literals emitted as ARRAY[...] using shared element handling)
- [x] BitString
- [x] Boolean
- [x] BoolExpr (AND/OR/NOT; precedence-aware parentheses preservation to maintain AST shape)
- [x] BooleanTest (IS TRUE/FALSE/UNKNOWN and negations)
- [x] CallContext (planner call metadata placeholder; no surface SQL output)
- [x] CallStmt (CALL procedure)
- [x] CaseExpr (CASE WHEN ... THEN ... ELSE ... END)
- [x] CaseTestExpr (planner CASE test placeholder emitted via case_test#type markers)
- [x] CaseWhen (WHEN condition THEN result)
- [x] CheckPointStmt (CHECKPOINT command)
- [x] ClosePortalStmt (CLOSE cursor|ALL)
- [x] ClusterStmt (CLUSTER [VERBOSE] table [USING index])
- [x] CoalesceExpr (COALESCE(...))
- [x] CoerceToDomain (domain coercion wrapper that defers to the inner expression)
- [x] CoerceToDomainValue (VALUE keyword inside domain check constraints)
- [x] CoerceViaIo (no-op cast via I/O that emits only the inner node)
- [x] CommentStmt (COMMENT ON object_type object IS comment with 42 object types)
- [x] ConstraintsSetStmt (SET CONSTRAINTS ALL|names DEFERRED|IMMEDIATE)
- [x] CopyStmt (COPY table/query TO/FROM file with options)
- [x] CollateClause (expr COLLATE collation_name, fixed to quote identifiers to preserve case)
- [x] CollateExpr (planner COLLATE wrapper emitting `coll#oid` placeholder when catalog names are unavailable)
- [x] ColumnDef (partial - column name, type, NOT NULL, DEFAULT, TODO: IDENTITY constraints, collation)
- [x] ColumnRef
- [x] CommonTableExpr (CTE definitions: name AS (query) for WITH clauses)
- [x] CompositeTypeStmt (CREATE TYPE ... AS (...))
- [x] CteCycleClause (WITH ... CYCLE clause with SET/USING targets)
- [x] CteSearchClause (WITH ... SEARCH {BREADTH|DEPTH} FIRST BY ... SET ...)
- [x] Constraint (all types: NOT NULL, DEFAULT, CHECK, PRIMARY KEY, UNIQUE, FOREIGN KEY, etc.)
- [x] ConvertRowtypeExpr (row-type coercions that forward to their argument)
- [x] CreateAmStmt (CREATE ACCESS METHOD name TYPE type HANDLER handler)
- [x] CreateCastStmt (CREATE CAST with source/target types, function, INOUT, context)
- [x] CreateConversionStmt (CREATE [DEFAULT] CONVERSION with encoding specifications)
- [x] CreatedbStmt (CREATE DATABASE)
- [x] CreateDomainStmt (CREATE DOMAIN)
- [x] CreateExtensionStmt (CREATE EXTENSION with IF NOT EXISTS and options)
- [x] CreateFdwStmt (CREATE FOREIGN DATA WRAPPER with handler and options)
- [x] CreateForeignServerStmt (CREATE SERVER with IF NOT EXISTS, TYPE, VERSION, FOREIGN DATA WRAPPER, OPTIONS)
- [x] CreateForeignTableStmt (CREATE FOREIGN TABLE with SERVER and OPTIONS)
- [x] CreateEnumStmt (CREATE TYPE ... AS ENUM, fixed to quote enum values)
- [x] CreateTableSpaceStmt (CREATE TABLESPACE name OWNER owner LOCATION 'path')
- [x] CreateEventTrigStmt (CREATE EVENT TRIGGER)
- [x] CreateFunctionStmt (CREATE FUNCTION/PROCEDURE with all options: AS, LANGUAGE, volatility, etc.)
- [x] CreateOpClassItem (helper for OPERATOR/FUNCTION/STORAGE items in CREATE OPERATOR CLASS)
- [x] CreateOpClassStmt (CREATE OPERATOR CLASS with DEFAULT, FOR TYPE, USING, FAMILY, AS items)
- [x] CreateOpFamilyStmt (CREATE OPERATOR FAMILY with USING access method)
- [x] CreatePLangStmt (CREATE LANGUAGE for procedural languages with HANDLER, INLINE, VALIDATOR)
- [x] CreatePolicyStmt (CREATE POLICY for row-level security with USING/WITH CHECK)
- [x] CreatePublicationStmt (CREATE PUBLICATION for logical replication with FOR ALL TABLES or specific objects)
- [x] CreateRangeStmt (CREATE TYPE AS RANGE with subtype and other parameters)
- [x] CreateSchemaStmt (CREATE SCHEMA with AUTHORIZATION and nested statements)
- [x] CreateSeqStmt (CREATE SEQUENCE)
- [x] CreateStatsStmt (CREATE STATISTICS on columns from tables)
- [x] StatsElem (CREATE STATISTICS column/expression entries)
- [x] CreateStmt (partial - basic CREATE TABLE, TODO: partitions, typed tables)
- [x] CreateSubscriptionStmt (CREATE SUBSCRIPTION for logical replication)
- [x] CreateTableAsStmt (CREATE TABLE ... AS ... / CREATE MATERIALIZED VIEW ... AS ...)
- [x] CreateTransformStmt (CREATE TRANSFORM FOR type LANGUAGE lang FROM/TO SQL WITH FUNCTION)
- [x] CreateTrigStmt (CREATE TRIGGER with BEFORE/AFTER/INSTEAD OF, timing, events, FOR EACH ROW/STATEMENT)
- [x] CreateUserMappingStmt (CREATE USER MAPPING FOR user SERVER server OPTIONS (...))
- [x] CurrentOfExpr (CURRENT OF cursor_name)
- [x] DeallocateStmt (DEALLOCATE prepared statement)
- [x] DeclareCursorStmt (DECLARE cursor FOR query)
- [x] DefElem (option name = value for WITH clauses)
- [x] DeleteStmt (DELETE FROM ... [USING ...] [WHERE ...] [RETURNING ...] with WITH clause support)
- [x] DiscardStmt (DISCARD ALL|PLANS|SEQUENCES|TEMP)
- [x] DistinctExpr (planner form of IS DISTINCT FROM emitted via deparse to recover operator tokens)
- [x] DoStmt (DO language block)
- [x] DropStmt (DROP object_type [IF EXISTS] objects [CASCADE])
- [x] DropOwnedStmt (DROP OWNED BY roles [CASCADE|RESTRICT])
- [x] DropRoleStmt (DROP ROLE [IF EXISTS] roles)
- [x] DropSubscriptionStmt (DROP SUBSCRIPTION [IF EXISTS] name [CASCADE|RESTRICT])
- [x] DropTableSpaceStmt (DROP TABLESPACE [IF EXISTS] name)
- [x] DropUserMappingStmt (DROP USER MAPPING FOR role SERVER server)
- [x] DropdbStmt (DROP DATABASE [IF EXISTS] name)
- [x] ExecuteStmt (EXECUTE prepared statement)
- [x] ExplainStmt (EXPLAIN (options) query)
- [x] FetchStmt (FETCH/MOVE cursor)
- [x] FieldSelect (composite field extraction wrapper that reuses the inner expression)
- [x] FieldStore (composite field assignment wrapper that reuses the inner expression)
- [x] Float
- [x] FromExpr (jointree helper for planner queries that walks FROM items and qualifiers with clause-aware wrapping)
- [x] FuncCall (comprehensive - basic function calls, special SQL standard functions with FROM/IN/PLACING syntax: EXTRACT, OVERLAY, POSITION, SUBSTRING, TRIM, TODO: WITHIN GROUP, FILTER)
- [x] FuncExpr (planner function invocation routed through the deparse bridge with placeholder `func#oid(...)` fallback)
- [x] FunctionParameter (CREATE FUNCTION parameters with mode keywords, identifiers, types, and DEFAULT clauses)
- [x] GrantStmt (GRANT/REVOKE privileges ON objects TO/FROM grantees, with options)
- [x] GrantRoleStmt (GRANT/REVOKE roles TO/FROM grantees WITH options GRANTED BY grantor)
- [x] GroupingFunc (GROUPING(columns) for GROUP BY GROUPING SETS)
- [x] GroupingSet (ROLLUP/CUBE/GROUPING SETS in GROUP BY clause)
- [x] ImportForeignSchemaStmt (IMPORT FOREIGN SCHEMA ... FROM SERVER ... INTO ...)
- [x] InferClause (ON CONFLICT target spec covering index columns or constraint references with optional WHERE predicate)
- [x] InferenceElem (planner ON CONFLICT inference element wrapper for index expressions)
- [x] InlineCodeBlock (inline DO block bodies rendered via dollar quoting)
- [x] IntoClause (SELECT/CREATE TABLE AS target with TEMP/UNLOGGED flags, reloptions, tablespace, and ON COMMIT handling)
- [x] IndexElem (index column with opclass, collation, ordering)
- [x] IndexStmt (CREATE INDEX with USING, INCLUDE, WHERE, etc.)
- [x] InsertStmt (WITH clause, column lists, OVERRIDING SYSTEM/USER VALUE, VALUES/SELECT/DEFAULT VALUES, ON CONFLICT, RETURNING)
- [x] Integer
- [x] IntList (integer list wrapper reused for planner metadata)
- [x] JoinExpr (all join types: INNER, LEFT, RIGHT, FULL, CROSS, with ON/USING clauses)
- [x] JsonFuncExpr (JSON_EXISTS, JSON_QUERY, JSON_VALUE functions - basic implementation)
- [x] JsonIsPredicate (IS JSON [OBJECT|ARRAY|SCALAR] predicates)
- [x] JsonParseExpr (JSON() function for parsing)
- [x] JsonScalarExpr (JSON_SCALAR() function)
- [x] JsonTable (JSON_TABLE() function with path, columns - basic implementation)
- [x] JsonArgument (PASSING clause arguments now reuse standalone emitter)
- [x] JsonBehavior (ON EMPTY/ON ERROR behavior keywords handled centrally)
- [x] JsonAggConstructor (shared aggregation tail for JSON_OBJECTAGG/JSON_ARRAYAGG covering ORDER BY, RETURNING, FILTER, and OVER clauses)
- [x] JsonExpr (SQL/JSON JSON_EXISTS/JSON_QUERY/JSON_VALUE emission with PASSING lists, wrappers, RETURNING, and ON EMPTY/ON ERROR handling)
- [x] JsonFormat (FORMAT/ENCODING clause helper for SQL/JSON output specs)
- [x] JsonOutput (RETURNING helper that emits type names plus optional format metadata)
- [x] JsonReturning (planner RETURNING metadata placeholder that keeps SQL/JSON expressions reparsable without catalog lookups)
- [x] JsonConstructorExpr (JSON_OBJECT/JSON_ARRAY constructors with RETURNING metadata placeholders)
- [x] JsonTableColumn (COLUMNS clause entries, including nested and wrapper/quotes controls)
- [x] JsonTablePath (placeholder for named JSON_TABLE path references)
- [x] JsonTablePathSpec (shared path specification with optional aliasing)
- [x] JsonTablePathScan (planner path scan placeholder with child plan emission)
- [x] JsonTableSiblingJoin (planner sibling join placeholder emitting child plans)
- [x] List (wrapper for comma-separated lists)
- [x] ListenStmt (LISTEN channel)
- [x] LoadStmt (LOAD 'library')
- [x] LockStmt (LOCK TABLE with lock modes)
- [x] LockingClause (SELECT ... FOR UPDATE/SHARE with OF targets and NOWAIT/SKIP LOCKED)
- [x] MergeStmt (MERGE INTO with WHEN MATCHED/NOT MATCHED clauses, supports UPDATE/INSERT/DELETE/DO NOTHING, WITH clause supported)
- [x] MergeAction (planner MERGE action placeholder tagged by match/command)
- [x] MergeWhenClause (WHEN branches shared across planner/parser MERGE flows)
- [x] MergeSupportFunc (planner merge support function placeholder emitting mergesupport#oid identifiers)
- [x] MinMaxExpr (GREATEST/LEAST functions)
- [x] MultiAssignRef (tuple-set assignments for UPDATE/ON CONFLICT `SET (...) = (...)` sequences)
- [x] NamedArgExpr (named arguments: name := value)
- [x] NextValueExpr (planner `nextval` placeholder emitted as `nextval#seqid` when sequence name is unavailable)
- [x] NotifyStmt (NOTIFY channel with optional payload)
- [x] NullTest (IS NULL / IS NOT NULL)
- [x] NullIfExpr (planner NULLIF variant forwarded through deparse to reconstruct function form)
- [x] ObjectWithArgs (function/operator names with argument types)
- [x] OidList (OID list wrapper reused for planner metadata)
- [x] OnConflictClause (ON CONFLICT DO NOTHING/DO UPDATE with target inference and optional WHERE clause)
- [x] OnConflictExpr (planner ON CONFLICT expression fallback with placeholder columns)
- [x] OpExpr (planner operator expression reconstructed via deparse to recover operator symbol)
- [x] Param (planner parameters emitted as param#kind:id placeholders)
- [x] ParamRef (prepared statement parameters $1, $2, etc.)
- [x] PlAssignStmt (PL/pgSQL assignment placeholder emitting `name := SELECT ...` when a backing query is present)
- [x] PartitionElem (column/expression in PARTITION BY clause with optional COLLATE and opclass)
- [x] PartitionCmd (ATTACH/DETACH PARTITION helper emitting FOR VALUES and CONCURRENTLY)
- [x] PartitionBoundSpec (FOR VALUES clause variants for RANGE/LIST/HASH partitions)
- [x] PartitionRangeDatum (MINVALUE/MAXVALUE/value helpers reused by partition bounds)
- [x] PartitionSpec (PARTITION BY RANGE/LIST/HASH with partition parameters)
- [x] SinglePartitionSpec (marker node for standalone PARTITION specifications)
- [x] PrepareStmt (PREPARE statement)
- [x] PublicationObjSpec (helper for CREATE/ALTER PUBLICATION object specifications)
- [x] PublicationTable (single-table publication entries with optional column lists and WHERE filters)
- [x] Query (planner Query node placeholder keyed by command type/source)
- [x] RawStmt (raw statement wrapper forwarding to the contained statement node)
- [x] RangeFunction (function calls in FROM clause, supports LATERAL, ROWS FROM, WITH ORDINALITY)
- [x] RangeSubselect (subquery in FROM clause, supports LATERAL)
- [x] RangeTableFunc (XMLTABLE() function with path and columns)
- [x] RangeTableSample (TABLESAMPLE with sampling method and REPEATABLE)
- [x] RangeTableFuncCol (XMLTABLE column definitions share planner/parser implementation)
- [x] RangeTblEntry (planner range table entries with kind-specific placeholders and child emission)
- [x] RangeTblFunction (planner RTE_FUNCTION entries with column list placeholders)
- [x] RangeTblRef (planner jointree references rendered as rte#index placeholders)
- [x] RtePermissionInfo (planner permission metadata placeholder emitting rteperm#relid plus column bitmap stats)
- [x] RangeVar (schema.table with optional alias support)
- [x] ReassignOwnedStmt (REASSIGN OWNED BY ... TO ...)
- [x] RefreshMatViewStmt (REFRESH MATERIALIZED VIEW)
- [x] RelabelType (implicit cast wrapper that leaves output unchanged)
- [x] ReindexStmt (REINDEX INDEX/TABLE/SCHEMA/DATABASE)
- [x] RenameStmt (ALTER ... RENAME TO ..., fixed to use rename_type field)
- [x] ReplicaIdentityStmt (REPLICA IDENTITY DEFAULT/FULL/NOTHING/USING INDEX)
- [x] ResTarget (partial - SELECT and UPDATE SET contexts)
- [x] TargetEntry (planner target wrapper forwarding expression plus optional alias)
- [x] RoleSpec (CURRENT_USER, SESSION_USER, CURRENT_ROLE, PUBLIC, role names)
- [x] RowCompareExpr (row-wise comparisons with tuple operators)
- [x] RowMarkClause (FOR lock clauses with wait policy placeholders)
- [x] RowExpr (ROW(...) or implicit row constructors)
- [x] RuleStmt (CREATE RULE ... AS ON ... TO ... DO ...)
- [x] ScalarArrayOpExpr (expr op ANY/ALL (array) constructs, converts to IN clause format)
- [x] SecLabelStmt (SECURITY LABEL FOR provider ON object_type object IS 'label')
- [x] SelectStmt (partial - SELECT with DISTINCT/DISTINCT ON, WINDOW clause definitions, LIMIT/OFFSET and FETCH WITH TIES, DML locking; VALUES for INSERT, WITH clause support)
- [x] SetOperationStmt (UNION/INTERSECT/EXCEPT with ALL support)
- [x] SetToDefault (DEFAULT keyword)
- [x] SortBy (ORDER BY expressions with ASC/DESC, NULLS FIRST/LAST, USING operator)
- [x] SortGroupClause (planner sort/group clause placeholder carrying operator OIDs and null ordering flags)
- [x] SqlValueFunction (CURRENT_DATE, CURRENT_TIME, CURRENT_TIMESTAMP, CURRENT_USER, etc.)
- [x] String (identifier and literal contexts)
- [x] SubLink (all sublink types: EXISTS, ANY, ALL, scalar subqueries, ARRAY)
- [x] SubPlan (planner subquery wrapper routed through deparse, falling back to its test expression)
- [x] SubscriptingRef (planner array subscripting with slice-aware `lower:upper` formatting and safe base parenthesisation)
- [x] AlternativeSubPlan (planner alternative subplan wrapper emitting first choice when deparse recovers nothing)
- [x] TableFunc (planner table function fallback for XMLTABLE/JSON_TABLE execution nodes)
- [x] TableSampleClause (planner TABLESAMPLE clause placeholder with handler OID args)
- [x] TableLikeClause (LIKE table_name for CREATE TABLE)
- [x] TruncateStmt (TRUNCATE table [RESTART IDENTITY] [CASCADE])
- [x] TriggerTransition (planner transition table placeholder for trigger execution)
- [x] TypeCast (CAST(expr AS type))
- [x] TypeName (canonicalises built-in names, decodes INTERVAL range/precision modifiers, handles array bounds)
- [x] UnlistenStmt (UNLISTEN channel)
- [x] UpdateStmt (UPDATE ... SET ... [FROM ...] [WHERE ...] [RETURNING ...] with WITH clause support)
- [x] VacuumRelation (table and columns for VACUUM)
- [x] VacuumStmt (partial - VACUUM/ANALYZE, basic implementation)
- [x] Var (planner Var nodes rendered as var#no^level.att placeholders)
- [x] VariableSetStmt (partial - SET variable = value, TODO: RESET, other variants)
- [x] VariableShowStmt (SHOW variable)
- [x] ViewStmt (CREATE [OR REPLACE] [TEMP] VIEW ... WITH (options) AS ... [WITH CHECK OPTION])
- [x] WindowDef (window specifications with frame clauses, offsets, and exclusion handling)
- [x] WindowClause (WINDOW clause definitions delegating to WindowDef formatting)
- [x] WindowFunc (planner window function nodes delegated through the deparse bridge with safety fallback)
- [x] WindowFuncRunCondition (planner run condition placeholder emitting comparison argument)
- [x] WithClause (WITH [RECURSIVE] for Common Table Expressions)
- [x] WithCheckOption (planner check option node emitted via deparse or raw qualifier when necessary)
- [x] XmlExpr (XMLELEMENT, XMLCONCAT, XMLCOMMENT, XMLFOREST, XMLPI, XMLROOT functions)
- [x] XmlSerialize (XMLSERIALIZE(DOCUMENT/CONTENT expr AS type))

## 📚 Implementation Learnings

Keep this section focused on durable guidance. When you add new insights, summarise them as short bullets and retire items that stop being relevant.

### Durable Guidance

**Enum Handling**:
- **Always use direct enum methods** instead of `TryFrom<i32>`: Call `n.field()` to get the typed enum rather than `TryFrom::try_from(n.field).ok()`. This eliminates fallible conversions and makes code cleaner.
- **Match on enums exhaustively**: Use proper enum variants in match statements instead of raw integer comparisons. Example: `match n.op() { SetOperation::SetopUnion => ... }` not `match n.op { 2 => ... }`.
- **Assert on unexpected enum values** instead of silently ignoring them. Use `assert!(false, "unexpected {}: {:?}", enum_name, value)` to fail fast on malformed ASTs.

**Assertions and Validation**:
- **Add strict assertions for expected argument counts** in special SQL functions (EXTRACT, OVERLAY, POSITION, etc.). Example: `assert!(n.args.len() == 2, "EXTRACT expects 2 arguments, got {}", n.args.len())`.
- Use `assert_node_variant!` instead of `if let Some(NodeEnum::...)` when you expect a specific type to catch bugs early.
- For `DefElem`-driven nodes, extract all fields first, then validate with assertions rather than silently falling through.

**Code Quality**:
- **Run `cargo clippy -p pgt_pretty_print` regularly** and fix warnings. Use `--fix --allow-dirty` to auto-fix most style issues.
- Avoid `TryFrom` patterns when the protobuf node provides direct accessor methods.
- Replace `if` chains with `match` for cleaner enum handling.
- Extend the snapshot harness' `clear_location` helper whenever new node families land so AST equality remains deterministic.

**String and Identifier Handling**:
- Reuse the helpers in `src/nodes/string.rs` for identifiers, keywords, and literals—avoid ad-hoc `TokenKind::IDENT` strings or manual quoting.
- Treat reserved keywords separately when deciding to quote identifiers; unreserved keywords like `name` can safely remain bare while true reserved words must stay quoted.

**Type Normalization**:
- Normalize TypeName built-ins by mapping `pg_catalog` identifiers to canonical SQL keywords while leaving user-defined schemas untouched.
- Decode INTERVAL typmods by interpreting the range bitmask in `typmods[0]` before emitting optional second precision so layouts like `INTERVAL DAY TO SECOND(3)` stay canonical.
- When the protobuf stores a single-component builtin (for example `bool` or `text`) without an explicit schema, keep the original casing and avoid reintroducing a `pg_catalog` qualifier so AST equality stays stable after reparse.

**Layout and Formatting**:
- Insert a `LineType::SoftOrSpace` breakpoint between join inputs and their qualifiers so long `ON` predicates can wrap without violating the target width while short joins stay single-line.
- Use `LineType::SoftOrSpace` before clause keywords such as `RENAME`, `OWNER`, and `SET SCHEMA` so long rename/owner statements wrap cleanly; rely on the line event for whitespace to avoid double spaces.
- Always wrap `ANY`/`ALL` right-hand operands in parentheses so the parser recognises them as subqueries or array expressions.
- Render symbolic operator names (composed purely of punctuation) without quoting and force a space before parentheses so DROP/ALTER statements remain parseable.
- Drop `LineType::SoftOrSpace` before optional DML clauses so compact statements stay single-line while long lists can wrap cleanly.
- Drop `LineType::SoftOrSpace` before `OVER` clauses and each window spec segment so inline window functions can wrap without blowing per-line limits while still re-parsing to the same AST.
- Preserve explicit parentheses in arithmetic expressions by wrapping child `AExpr` nodes whenever their operator precedence is lower than the parent or a left-associative parent holds a right-nested operand; otherwise constructs like `100 * 3 + (vs.i - 1) * 3` lose grouping and fail AST equality.
- Let `emit_clause_condition` drive wrapping for WHERE/HAVING predicates by emitting a `SoftOrSpace` break plus indentation; this keeps long filters under the line budget without sprinkling manual spaces.
- `FromExpr` emitters should stick to list helpers for FROM items and rely on `emit_clause_condition` for qualifiers—the surrounding statement is responsible for injecting the `FROM` keyword.
- Wrap `BoolExpr` children whose precedence is lower than their parent (e.g. OR under AND, AND/OR under NOT) so expressions like `(a OR b) AND c` retain explicit parentheses and keep the original AST structure.
- Use `emit_clause_condition` to indent boolean clause bodies (`WHERE`, `HAVING`, planner filters) so wrapped predicates align under their keywords instead of hugging the left margin.
- Emit CTE SEARCH/CYCLE clauses using `LineType::SoftOrSpace` so they stay attached to the CTE block while breaking cleanly when alias lists grow.
- Introduce `LineType::SoftOrSpace` breaks before `PARTITION OF`, `FOR VALUES`, `PARTITION BY`, and `ATTACH/DETACH PARTITION` clauses so long partition DDL respects 60-character budgets without sacrificing single-line output when space allows.

**Node-Specific Patterns**:
- Respect `AIndices::is_slice`; emit the colon only when the slice flag is set so single-element subscripts (e.g. `col[1]`) retain their original structure.
- Parenthesise AIndirection bases that are not plain `ColumnRef`, `ParamRef`, or nested indirections whenever subscripts are present so casts, function calls, and literals remain parseable when indexed.
- Emit `DO` bodies before the optional `LANGUAGE` clause to preserve `DefElem` ordering in the resulting AST.
- Respect `CoercionForm` when emitting row constructors; implicit casts must stay bare tuples or the planner-visible `row_format` flag changes.
- When emitting CTE materialization hints, match on `CteMaterialize::Always`/`::Never` to emit the hint; default CTEs should not emit any materialization keyword.
- Map `SelectStmt::limit_option` to `FETCH ... WITH TIES` when it resolves to `LimitOption::WithTies` so the re-parsed AST retains the original limit semantics.
- When wrapping a `SelectStmt` inside outer statements (e.g. VIEW, COPY), emit it via `emit_select_stmt_no_semicolon` so trailing clauses can follow before the final semicolon.
- Planner `SubscriptingRef` nodes expose slice syntax via `reflowerindexpr`; emit a colon whenever that slot exists (even if the bound is NULL) so constructs like `arr[:upper]` and `arr[lower:]` retain their shape.
- Planner `CollateExpr` only carries a collation OID; emit `coll#oid` placeholders and treat `coll_oid == 0` as `COLLATE DEFAULT` so round-trips stay valid without catalog lookups.
- `IntoClause::relpersistence` uses `'t'`/`'u'` for TEMP/UNLOGGED; decode `OnCommitAction` into `PRESERVE ROWS`, `DELETE ROWS`, or `DROP` and reuse the options helper for `WITH (...)` lists.
- Decode window frame bitmasks to render RANGE/ROWS/GROUPS with the correct UNBOUNDED/CURRENT/OFFSET bounds and guard PRECEDING/FOLLOWING against missing offsets.
- Ordered-set aggregates must render `WITHIN GROUP (ORDER BY ...)` outside the argument list and emit `FILTER (WHERE ...)` ahead of any `OVER` clause so planner fallbacks reuse the same surface layout.
- For `MergeStmt`, only append `BY TARGET` when the clause has no predicate (the `DO NOTHING` branch); conditional branches should stay as bare `WHEN NOT MATCHED` so we don't rewrite user intent.
- Collapse `MultiAssignRef` clusters by reading `ncolumns` and formatting `(col1, col2, ...) = (expr1, expr2, ...)` once, then skip the trailing ResTargets to avoid duplicate output; if the source row is a single `ROW(...)` expression (e.g. `ROW(excluded.*)`), emit it directly instead of forcing an expanded tuple so the AST and semantics stay aligned.
- Treat `BEGIN` followed by transaction modifiers (`TRANSACTION`, isolation keywords, READ/WRITE, DEFERRABLE flags) as a standalone statement in the splitter; only procedural `BEGIN … END` blocks should bump the nesting depth.
- Map `transaction_*` defelems to idiomatic SQL (`ISOLATION LEVEL`, `READ ONLY`/`READ WRITE`, `DEFERRABLE`/`NOT DEFERRABLE`) when emitting `TransactionStmt` options so the formatted output reparses successfully.
- When a binary comparison must wrap, keep the operator attached to the left expression and indent the right-hand side behind a `LineType::SoftOrSpace` break. This avoids the renderer splitting each token onto its own line once the surrounding group has already broken.
- Planner-only emitters should stick to quoted placeholders like `rte#kind` or `jsonpathscan[...]` via `emit_identifier` so the renderer keeps non-surface nodes reparsable without catalog lookups.

- Decode trigger timing bitmasks before emitting keywords: check `INSTEAD` bits first, treat `BEFORE` as the explicit flag, and fall back to `AFTER` when no timing bits are set so reparse preserves the original `CreateTrigStmt::timing`.
- Index expressions in `IndexElem` must always be wrapped in parentheses—PostgreSQL syntax requires `CREATE INDEX ... ON table ((expression))` not `CREATE INDEX ... ON table (expression)`.
- For index-related ALTER TABLE commands (e.g., `ALTER INDEX ... ALTER COLUMN 1 SET STATISTICS`), column references use the `num` field for numeric positions, not the `name` field.
- `TableLikeClause` options are stored as a bitmap where `0x7FFFFFFF` (all 31 bits set) represents `INCLUDING ALL`; individual bits correspond to INCLUDING DEFAULTS (1<<0), INCLUDING CONSTRAINTS (1<<1), INCLUDING IDENTITY (1<<2), etc.
- When protobuf fields like `relation_type` contain unexpected values (e.g., `ObjectAccessMethod` for a table constraint), validate against actual data fields like `relation.relname` to determine the correct object type—the protobuf enum can be misleading.

**Planner Nodes (CRITICAL - Read Carefully)**:
- **NEVER create synthetic nodes or wrap nodes in SELECT statements for deparse round-trips**. This violates the architecture and breaks AST preservation.
- **NEVER call `pgt_query::deparse()` from emit functions**. The pretty printer must emit directly from AST nodes.
- Planner nodes (OpExpr, Aggref, WindowFunc, FuncExpr, SubPlan, etc.) represent internal PostgreSQL optimizer structures with OIDs instead of names.
- For planner nodes, emit simple fallback representations using OID placeholders (e.g., `op#123`, `func#456`, `agg#789`).
- Example: `OpExpr` with args `[a, b]` and `opno=96` emits as `a op#96 b` - we don't have operator symbols without a catalog lookup.
- `DistinctExpr` can emit `IS DISTINCT FROM` since the syntax is known; `NullIfExpr` can emit `NULLIF(a, b)` for the same reason.
- Planner nodes indicate the pretty printer was given optimizer output rather than parser output - the fallback representations are acceptable.
- When duplicating window frame logic between `WindowClause` and `WindowDef`, **copy and adapt the code directly** rather than creating synthetic nodes or calling helper functions that expect different node types.
- When a placeholder string includes punctuation (`op#`, `var#`, etc.), emit it via `emit_identifier` so the fallback stays valid SQL and reparses cleanly.

### Logging Future Work
- Capture new learnings as concise bullets here and keep detailed session history in commit messages or external notes.
- Track open follow-ups (e.g. resolving operator lookups for `ScalarArrayOpExpr`) in the "Next Steps" section instead of long-form logs.

## Code Generation

The project uses procedural macros for code generation:

- **TokenKind**: Generated from keywords and operators
- **GroupKind**: Generated for each node type

If you need to add new tokens or groups:

1. Check if code generation is needed (usually not for individual nodes)
2. Tokens are likely already defined for all SQL keywords
3. Groups are auto-generated based on node types

## References

### Key Files
- `src/nodes/mod.rs`: Central dispatch for all node types
- `src/nodes/select_stmt.rs`: Example of complex statement
- `src/nodes/a_expr.rs`: Example of expression handling
- `src/nodes/node_list.rs`: List helper functions
- `parser/ast/statements.go`: Go reference for statements
- `parser/ast/expressions.go`: Go reference for expressions

### Useful Commands
```bash
# Run formatter on all code
just format

# Run all tests
just test

# Run specific crate tests
cargo test -p pgls_pretty_print

# Update test snapshots
cargo insta review

# Run clippy
just lint

# Check if ready to commit
just ready

# Run agentic task
just agentic pretty_printer
```

## Next Steps

1. Capture focused SQL/JSON fixtures (JSON_EXISTS/JSON_QUERY/JSON_VALUE with PASSING, RETURNING, wrappers) to validate the new `JsonExpr` and aggregation emitters.
2. Add a compact partition DDL fixture that exercises RANGE/LIST bounds to lock in `PartitionBoundSpec` and `PartitionRangeDatum` output.
3. Curate fixtures for the new planner placeholders (`MergeSupportFunc`, `RtePermissionInfo`, `SortGroupClause`, `PlAssignStmt`) and accept snapshots via `cargo insta review`.
4. Re-run the audit for remaining `todo!("emit_node_enum" ...)` fallbacks to confirm the outstanding node list for the next implementation batch.

## ⚠️ CRITICAL: When to Signal Completion

**When ALL of the following are true:**
- ✅ All ~270 AST nodes are implemented (check "Completed Nodes" section)
- ✅ All tests are passing (`cargo test -p pgls_pretty_print` shows 0 failures)
- ✅ There is absolutely nothing left to do
- ✅ The codebase is ready for final review

**Output this exact keyword on its own line:** `===AGENTIC_TASK_COMPLETE===`

This signals that the implementation is 100% finished.

## Summary: Key Points

### ✅ DO:
- **Implement `emit_*` functions** for AST nodes in `src/nodes/`
- **Create test cases** to validate your implementations
- **Run specific tests** with `cargo test -p pgt_pretty_print test_single__<name> -- --show-output`
- **Implement nodes partially** - handle common fields first, add TODOs for the rest
- **Use Go parser** as reference for SQL generation logic
- **Use pgFormatter for inspiration** on line breaking: `pg_format tests/data/single/your_test.sql`
- **Use existing helpers** from `node_list.rs` for lists
- **Use `assert_node_variant!`** to extract specific node types from generic Nodes
- **⚠️ UPDATE THIS DOCUMENT** after each session:
  - Mark nodes as `[x]` in "Completed Nodes"
  - Refresh the bullets under "Implementation Learnings" (add new guidance, remove stale notes)
  - Update progress or pending work in "Next Steps"

### ❌ DON'T:
- **Don't modify** `src/renderer.rs` (layout engine - complete)
- **Don't modify** `src/emitter.rs` (event emitter - complete)
- **Don't modify** `tests/tests.rs` (test infrastructure - complete)
- **Don't modify** `src/codegen/` (code generation - complete)
- **Don't try to implement everything at once** - partial implementations are fine!
- **NEVER create synthetic AST nodes** or wrap nodes in SELECT for deparse round-trips
- **NEVER call `pgt_query::deparse()`** from emit functions - emit directly from AST
- **NEVER create new node instances** to reuse helpers - copy/adapt code directly instead

### 🎯 Goals:
- **~270 total nodes** to eventually implement
- **~14 nodes** currently done
- **~50 high-priority nodes** should be tackled first
- **Each node** can be implemented incrementally
- **Tests validate** both correctness (AST equality) and formatting (line length)

## Notes

- The pretty printer is **structure-preserving**: it should not change the AST
- The formatter is **line-length-aware**: it respects `max_line_length` when possible
- String literals and JSON content may exceed line length (allowed by tests)
- The renderer uses a **greedy algorithm**: tries single-line first, then breaks
- Groups enable **local layout decisions**: inner groups can break independently

## Quick Reference: Adding a New Node

Follow these steps to implement a new AST node:

### 1. Create the file

```bash
# Create new file in src/nodes/
touch src/nodes/<node_name>.rs
```

### 2. Implement the emit function

```rust
// src/nodes/<node_name>.rs
use pgt_query::protobuf::<NodeType>;
use crate::{TokenKind, emitter::{EventEmitter, GroupKind}};

pub(super) fn emit_<node_name>(e: &mut EventEmitter, n: &<NodeType>) {
    e.group_start(GroupKind::<NodeName>);

    // Emit tokens, spaces, and child nodes
    e.token(TokenKind::KEYWORD_KW);
    e.space();
    // ... implement based on Go SqlString() method

    e.group_end();
}
```

### 3. Register in mod.rs

```rust
// src/nodes/mod.rs

// Add module declaration
mod <node_name>;

// Add import
use <node_name>::emit_<node_name>;

// Add to dispatch in emit_node_enum()
pub fn emit_node_enum(node: &NodeEnum, e: &mut EventEmitter) {
    match &node {
        // ... existing cases
        NodeEnum::<NodeName>(n) => emit_<node_name>(e, n),
        // ...
    }
}
```

### 4. Test

```bash
# Run tests to see if it works
cargo test -p pgt_pretty_print

# Review snapshot output
cargo insta review
```

### 5. Iterate

- Check Go implementation in `parser/ast/*.go` for reference
- Adjust groups, spaces, and line breaks based on test output
- Ensure AST equality check passes (tests validate this automatically)

## Files You'll Work With

**Primary files** (where you implement):
- `src/nodes/mod.rs` - Register new nodes here
- `src/nodes/<node_name>.rs` - Implement each node here
- `src/nodes/node_list.rs` - Helper functions (read-only, may add helpers)
- `src/nodes/string.rs` - String/identifier helpers (read-only)

**Reference files** (read for examples):
- `src/nodes/select_stmt.rs` - Complex statement example
- `src/nodes/update_stmt.rs` - Example with `assert_node_variant!`
- `src/nodes/res_target.rs` - Example with multiple emit functions
- `src/nodes/range_var.rs` - Simple node example
- `src/nodes/column_ref.rs` - List helper example

**Go reference files** (read for SQL logic):
- `parser/ast/statements.go` - Main SQL statements
- `parser/ast/expressions.go` - Expression nodes
- `parser/ast/ddl_statements.go` - DDL statements
- Other `parser/ast/*.go` files as needed

**DO NOT MODIFY**:
- `src/renderer.rs` - Layout engine (already complete)
- `src/emitter.rs` - Event emitter (already complete)
- `src/codegen/` - Code generation (already complete)
- `tests/tests.rs` - Test infrastructure (already complete)

## 📝 Session Summaries

**Session history has been moved to [session_log.md](./session_log.md)** for easier maintenance.

To add a new session entry, update [session_log.md](./session_log.md) using this template:

```markdown
---
**Date**: YYYY-MM-DD (Session N)
**Nodes Implemented/Fixed**: [List of nodes]
**Progress**: X/270 → Y/270
**Tests**: N passed (was M)
**Key Changes**:
- [Bullet list of important changes]

**Learnings**:
- [New patterns discovered]
- [Bugs fixed]
- [API changes]

**Next Steps**:
- [What to tackle next]
- [Known issues to address]
---
```

**Instructions**:
1. Add new session summaries at the TOP of session_log.md (most recent first)
2. Keep summaries concise - focus on what changed and why
3. Reference specific files and line numbers when useful
4. Move durable insights up to "Durable Guidance" section in this file


## Notes

- The pretty printer is **structure-preserving**: it should not change the AST
- The formatter is **line-length-aware**: it respects `max_line_length` when possible
- String literals and JSON content may exceed line length (allowed by tests)
- The renderer uses a **greedy algorithm**: tries single-line first, then breaks
- Groups enable **local layout decisions**: inner groups can break independently

## Quick Reference: Adding a New Node

Follow these steps to implement a new AST node:

### 1. Create the file

```bash
# Create new file in src/nodes/
touch src/nodes/<node_name>.rs
```

### 2. Implement the emit function

```rust
// src/nodes/<node_name>.rs
use pgt_query::protobuf::<NodeType>;
use crate::{TokenKind, emitter::{EventEmitter, GroupKind}};

pub(super) fn emit_<node_name>(e: &mut EventEmitter, n: &<NodeType>) {
    e.group_start(GroupKind::<NodeName>);

    // Emit tokens, spaces, and child nodes
    e.token(TokenKind::KEYWORD_KW);
    e.space();
    // ... implement based on Go SqlString() method

    e.group_end();
}
```

### 3. Register in mod.rs

```rust
// src/nodes/mod.rs

// Add module declaration
mod <node_name>;

// Add import
use <node_name>::emit_<node_name>;

// Add to dispatch in emit_node_enum()
pub fn emit_node_enum(node: &NodeEnum, e: &mut EventEmitter) {
    match &node {
        // ... existing cases
        NodeEnum::<NodeName>(n) => emit_<node_name>(e, n),
        // ...
    }
}
```

### 4. Test

```bash
# Run tests to see if it works
cargo test -p pgt_pretty_print

# Review snapshot output
cargo insta review
```

### 5. Iterate

- Check Go implementation in `parser/ast/*.go` for reference
- Adjust groups, spaces, and line breaks based on test output
- Ensure AST equality check passes (tests validate this automatically)

## Files You'll Work With

**Primary files** (where you implement):
- `src/nodes/mod.rs` - Register new nodes here
- `src/nodes/<node_name>.rs` - Implement each node here
- `src/nodes/node_list.rs` - Helper functions (read-only, may add helpers)
- `src/nodes/string.rs` - String/identifier helpers (read-only)

**Reference files** (read for examples):
- `src/nodes/select_stmt.rs` - Complex statement example
- `src/nodes/update_stmt.rs` - Example with `assert_node_variant!`
- `src/nodes/res_target.rs` - Example with multiple emit functions
- `src/nodes/range_var.rs` - Simple node example
- `src/nodes/column_ref.rs` - List helper example

**Go reference files** (read for SQL logic):
- `parser/ast/statements.go` - Main SQL statements
- `parser/ast/expressions.go` - Expression nodes
- `parser/ast/ddl_statements.go` - DDL statements
- Other `parser/ast/*.go` files as needed

**DO NOT MODIFY**:
- `src/renderer.rs` - Layout engine (already complete)
- `src/emitter.rs` - Event emitter (already complete)
- `src/codegen/` - Code generation (already complete)
- `tests/tests.rs` - Test infrastructure (already complete)
