# Autonomous PostgreSQL Pretty Printer Implementation and Formatting

You are an autonomous Rust code generator building and improving a PostgreSQL pretty printer by implementing ToTokens traits for AST nodes and ensuring all output is properly formatted.

## REQUIRED READING:
Before starting, you MUST read these additional files in the agentic directory for complete context and guidelines:

1. **`agentic/sql_formatter_improvements.md`** - Contains detailed formatting principles, EventEmitter usage patterns, and code examples
2. **`agentic/fix_remaining_formatter_failures.md`** - Contains the comprehensive workflow for reviewing and fixing all formatter tests systematically

These files contain critical information about formatting rules, indentation hierarchy, line breaking strategies, and testing workflows that are essential for this task.

## PROJECT CONTEXT:
This is a PostgreSQL pretty printer that takes parsed AST nodes and converts them back into well-formatted SQL code. Many AST nodes already have ToTokens implementations, but they need to be improved to produce better formatting.

Your goal is to:
1. Implement ToTokens for any remaining unimplemented nodes
2. Fix and improve existing implementations to produce properly formatted output
3. Ensure all tests pass with correctly formatted SQL that respects line length limits

## YOUR INFINITE LOOP TASK:

1. **Run all pretty printer tests** to identify failing tests: `cargo test -p pgt_pretty_print`
2. **Pick the next failing test** from the output
3. **Examine the test file** in `crates/pgt_pretty_print/tests/data/` to understand the SQL
4. **Use pgFormatter for inspiration**: `pg_format crates/pgt_pretty_print/tests/data/[test_file].sql`
5. **Read the current snapshot** if it exists to see the current output
6. **Find the relevant ToTokens implementation** in `crates/pgt_pretty_print/src/nodes.rs`
7. **Improve the formatting**:
   - Add proper line breaks with
   - Add indentation for subordinate clauses
   - Ensure line length compliance (60 chars max)
   - Follow the indentation hierarchy rules
8. **Run the specific test** to check your changes
9. **Read the updated snapshot** to verify the output looks correct
10. **Run all tests again** to ensure no regressions
11. **Repeat from step 1** until all tests pass

## CRITICAL RULES:

- **NEVER EVER add comments to Rust code** - ZERO // comments, ZERO /* */ comments in implementations
- **NEVER manually implement nested nodes** - Always call .to_tokens(e) on child nodes, never implement their logic
- **Use existing ToTokens implementations** - If a node already has ToTokens, call it, don't reimplement
- **ALWAYS use enum accessor methods** - Use `self.field()` NOT `EnumType::try_from(self.field).unwrap()` for enum fields
  - Example: `self.jointype()` returns `JoinType` enum directly
  - Example: `self.op()` returns the operation enum directly
  - Never use manual try_from conversions for protobuf enum fields
- **Line length must be ≤ 60 characters** (except for JSON content)
- **JSON Exception**: Do NOT format JSON strings/objects - accept longer line widths for JSON content
- **Use pgFormatter as inspiration** - Run `pg_format` on test files to see ideal formatting

## GROUP AND CONTEXT SYSTEM:

- **Complex nodes MUST create groups**: `e.group_start(GroupKind::NodeName, None, false);` and `e.group_end();`
- **Simple nodes NO groups**: String, AConst, and other leaf nodes emit tokens directly
- **Context-aware formatting**: Use `e.is_within_group(GroupKind::ParentType)` to adapt behavior
- **Semicolons for top-level statements**: Use `if e.is_top_level() { e.token(TokenKind::SEMICOLON); }`
- **Never maintain explicit context state** - always introspect the event stream

## FORBIDDEN PATTERNS:
- `// Handle XYZ formatting directly` - NEVER add explanatory comments
- `if let Some(node::Node::SomeType(inner)) = &node.node { /* manual implementation */ }` - NEVER manually implement child nodes
- Always use: `child_node.to_tokens(e)` instead of manual implementations

## FILE PATHS:
- Target file for implementations: `crates/pgt_pretty_print/src/nodes.rs`
- Single-statement test files: `crates/pgt_pretty_print/tests/data/single/*.sql`
- Multi-statement test files: `crates/pgt_pretty_print/tests/data/multi/*.sql`
- Snapshot files: `crates/pgt_pretty_print/tests/snapshots/tests__test_formatter__*.snap`
- AST structure reference: `crates/pgt_query/src/protobuf.rs`

## TOOLS AVAILABLE:
- Read: Read any file
- Write: Create new files
- Edit/MultiEdit: Modify existing files
- Bash: Run commands like cargo check, cargo test
- Grep: Search for patterns in files
- ast-grep: Use ast-grep (https://ast-grep.github.io/) for advanced code pattern searching and analysis

## FORMATTING PRINCIPLES:

### Indentation Hierarchy
1. **Main statement keywords** (CREATE, ALTER, MERGE, etc.) - Base level (no indent)
2. **Primary clauses** (SELECT, FROM, WHERE, etc.) - Base level
3. **Optional/subordinate clauses** (TO, DO, OPTIONS, VERSION, etc.) - Indented 1 level
4. **Sub-clauses within actions** (SET assignments, VALUES lists) - Indented 2 levels
5. **Complex nested structures** - Additional indentation as needed

### Line Breaking Strategy
- Use `LineType::SoftOrSpace` or `LineType::Soft` before major clauses
- Keep related parts together when possible
- Break at natural clause boundaries (WHERE, ORDER BY, GROUP BY)
- Respect 60 character line limit (except JSON content)

### Context-Aware Formatting
Use EventEmitter context helpers when needed:
- `e.is_within_group(GroupKind::SelectStmt)` - Check if within a specific statement type
- `e.parent_group()` - Get immediate parent group for conditional formatting
- Most formatting should be consistent regardless of context

## TESTING WORKFLOW:

1. **Run all tests to find failures**:
   ```bash
   cargo test -p pgt_pretty_print
   ```

2. **Run specific failing test**:
   ```bash
   cargo test -p pgt_pretty_print test_formatter__[test_name] -- --show-output
   ```

3. **Use pgFormatter for ideal formatting**:
   ```bash
   pg_format crates/pgt_pretty_print/tests/data/[test_file].sql
   ```

4. **Review snapshot after changes**:
   ```bash
   cat crates/pgt_pretty_print/tests/snapshots/tests__test_formatter__[test_name].snap
   ```

5. **Accept valid changes**:
   ```bash
   cargo insta review
   ```

## CORRECT IMPLEMENTATION PATTERN:
```rust
// Complex node with group
impl ToTokens for SomeComplexNode {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SomeComplexNode, None, false);

        // Using enum accessor methods - ALWAYS use this pattern
        match self.jointype() {  // ✅ CORRECT - use accessor method
            JoinType::JoinInner => e.token(TokenKind::INNER_KW),
            JoinType::JoinLeft => e.token(TokenKind::LEFT_KW),
            _ => {}
        }

        e.token(TokenKind::KEYWORD("SELECT".to_string()));
        if let Some(ref child) = self.some_child {
            child.to_tokens(e);  // ✅ CORRECT - delegate to existing ToTokens
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

// Simple leaf node without group
impl ToTokens for SomeSimpleNode {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.value.clone()));
    }
}

// Context-aware node
impl ToTokens for ContextSensitiveNode {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ContextSensitiveNode, None, false);

        if e.is_within_group(GroupKind::UpdateStmt) {
            // UPDATE SET formatting: column = value
        } else {
            // SELECT formatting: value AS alias
        }

        e.group_end();
    }
}
```

## WRONG IMPLEMENTATION PATTERN (FORBIDDEN):
```rust
impl ToTokens for SomeNode {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SomeNode, None, false);

        // ❌ FORBIDDEN - using try_from for enum conversion
        match JoinType::try_from(self.jointype).unwrap() {
            JoinType::JoinInner => e.token(TokenKind::INNER_KW),
            _ => {}
        }

        if let Some(ref child) = self.some_child {
            // Handle child formatting directly ❌ FORBIDDEN COMMENT
            if let Some(node::Node::ChildType(inner)) = &child.node {
                // ❌ FORBIDDEN - manual implementation of child
                e.token(TokenKind::IDENT(inner.name.clone()));
            }
        }

        e.group_end();
    }
}
```

## COMMON FORMATTING PATTERNS:

### ALTER Statements
```rust
// Always indent optional clauses
e.indent_start();
e.line(LineType::SoftOrSpace);
e.token(TokenKind::VERSION_KW);
e.space();
version.to_tokens(e);
e.indent_end();
```

### Lists and Parameters
```rust
for (i, item) in items.iter().enumerate() {
    if i > 0 {
        e.token(TokenKind::COMMA);
        e.line(LineType::SoftOrSpace);  // Allow breaking between items
    }
    item.to_tokens(e);
}
```

### Hierarchical Structures (MERGE, etc.)
```rust
// Indent actions under WHEN clauses
e.indent_start();  // Level 1
e.line(LineType::SoftOrSpace);
e.token(TokenKind::UPDATE_KW);
e.space();
e.token(TokenKind::SET_KW);
e.indent_start();  // Level 2 for SET items
// ...
e.indent_end();    // End level 2
e.indent_end();    // End level 1
```

## KEY CONTEXT RULES:
- **Complex nodes** (statements, expressions, structured data) MUST use groups with GroupKind
- **Simple leaf nodes** (String, AConst) emit tokens directly without groups
- **Context-sensitive nodes** use `e.is_within_group(GroupKind::ParentType)` to adapt formatting
- **Top-level statements** use `e.is_top_level()` to add semicolons
- **Always delegate** to existing ToTokens implementations

## ACTION PLAN:
1. Start by running all tests to see the current state
2. Focus on tests that fail due to line length violations or poor formatting
3. Systematically improve each ToTokens implementation
4. Verify each change produces better formatted output
5. Continue until all tests pass with properly formatted SQL

## Completion Signal Requirements

**CRITICAL**: ONLY when ALL tests pass and formatting is complete, output this exact phrase:

`TASK COMPLETE`

This allows automated scripts to detect task completion reliably.

If not ALL tests pass or there is formatting work left to do, output `CONTINUE` instead.

Continue this loop indefinitely until all tests pass with properly formatted output.
