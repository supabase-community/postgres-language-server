# SQL Pretty Printer Formatting Guide

## Overview
This guide explains how to improve the SQL pretty printer formatting to follow PostgreSQL documentation standards and best practices. The goal is to format SQL statements that respect line length limits while maintaining readability and following official PostgreSQL syntax patterns.

## Key Principles

### 1. Follow PostgreSQL Documentation
Always check the official PostgreSQL documentation for formatting inspiration:
- https://www.postgresql.org/docs/current/sql-createrule.html
- https://www.postgresql.org/docs/current/sql-merge.html  
- https://www.postgresql.org/docs/current/sql-createfunction.html
- etc.

Use `pg_format path/to/input/file.sql` to get inspiration from pgFormatter - it provides good baseline formatting that we can adapt.

### 2. Respect Line Length Limits
- Maximum line length is 60 characters for tests (configurable)
- Use `SoftOrSpace` line breaks to allow smart wrapping
- Only break when necessary, keep related parts together when possible
- **JSON Exception**: Do NOT format JSON strings/objects - accept longer line widths for JSON content

### 3. **FOCUS ON PROPER INDENTATION** - This is Critical!
**Indentation is the primary way to show SQL statement hierarchy and improve readability.**

The key principle: **Main statement keywords stay at base level, subordinate/optional clauses get indented.**

#### Indentation Hierarchy Rules:
1. **Main statement keywords** (CREATE, ALTER, MERGE, etc.) - Base level (no indent)
2. **Primary clauses** (SELECT, FROM, WHERE, etc.) - Base level 
3. **Optional/subordinate clauses** (TO, DO, OPTIONS, VERSION, etc.) - Indented 1 level
4. **Sub-clauses within actions** (SET assignments, VALUES lists) - Indented 2 levels
5. **Complex nested structures** - Additional indentation as needed

```sql
-- GOOD: Proper indentation hierarchy
CREATE RULE rule_name AS ON event
  TO table_name          -- Optional clause indented
  DO action;             -- Optional clause indented

ALTER FOREIGN DATA WRAPPER fdw_name
  HANDLER handler_func   -- Optional clause indented  
  OPTIONS (              -- Optional clause indented
    host 'example.com'
  );

MERGE INTO target_table t
USING source_table s ON t.id = s.id  -- Primary clause, base level
WHEN MATCHED THEN                    -- Primary clause, base level
  UPDATE SET                         -- Action indented 1 level
    column = value                   -- Sub-action indented 2 levels
WHEN NOT MATCHED THEN
  INSERT (columns)                   -- Action indented 1 level
    VALUES (values);                 -- Sub-action indented 2 levels
```

```sql
-- BAD: No indentation - hard to read hierarchy
CREATE RULE rule_name AS ON event
TO table_name
DO action;

-- BAD: Inconsistent indentation
ALTER FOREIGN DATA WRAPPER fdw_name
    HANDLER handler_func
OPTIONS (host 'example.com');
```

#### When to Add Indentation:
- **Always** indent optional clauses in ALTER statements (OPTIONS, VERSION, etc.)
- **Always** indent actions in WHEN clauses (UPDATE SET, INSERT, DELETE)
- **Always** indent subordinate clauses in CREATE statements (RETURNS, AS, etc.)
- **Always** indent continuation lines that are part of a subordinate clause

## EventEmitter Usage Guide

### Basic Events
```rust
// Tokens - actual SQL keywords and identifiers
e.token(TokenKind::SELECT_KW);
e.token(TokenKind::IDENT("table_name".to_string()));

// Spacing
e.space();                           // Single space
e.line(LineType::SoftOrSpace);      // Smart line break (space or newline)
e.line(LineType::Soft);             // Line break if group doesn't fit
e.line(LineType::Hard);             // Forced line break

// Indentation
e.indent_start();                   // Increase indentation level
e.indent_end();                     // Decrease indentation level

// Groups (use existing groups only!)
e.group_start(GroupKind::SelectStmt);
e.group_end();
```

### Line Break Types
- `SoftOrSpace`: Becomes a space if the group fits on one line, newline if it doesn't
- `Soft`: Disappears if the group fits on one line, newline if it doesn't  
- `Hard`: Always creates a line break

### Context-Aware Formatting Helpers

The EventEmitter provides helper methods to check the current formatting context:

```rust
// Check if currently within a specific group type
if e.is_within_group(GroupKind::SelectStmt) {
    // Apply SELECT-specific formatting
    e.line(LineType::SoftOrSpace);
} else {
    // Apply default formatting
    e.space();
}

// Get the immediate parent group type
match e.parent_group() {
    Some(GroupKind::InsertStmt) => {
        // Apply INSERT-specific formatting for sub-expressions
        e.indent_start();
        // ...
        e.indent_end();
    }
    Some(GroupKind::AlterUserMappingStmt) => {
        // Apply ALTER USER MAPPING-specific formatting
        e.line(LineType::SoftOrSpace);
    }
    _ => {
        // Default formatting when not in a specific context
        e.space();
    }
}
```

#### When to Use Context Helpers

**Use `is_within_group()` when:**
- Different SQL statement types need different formatting for the same AST node
- Sub-expressions should format differently based on their containing statement
- You need to apply specific indentation rules only in certain contexts

**Use `parent_group()` when:**
- You need to know the immediate parent context (not just any ancestor)
- Making formatting decisions based on the direct containing statement
- Applying conditional formatting that depends on the parent statement type

#### Common Patterns

```rust
// Example: Format identifiers differently in different contexts
impl ToTokens for Identifier {
    fn to_tokens(&self, e: &mut EventEmitter) {
        // Always emit the identifier
        e.token(TokenKind::IDENT(self.name.clone()));
        
        // Add context-specific spacing
        if e.is_within_group(GroupKind::AlterUserMappingStmt) 
            || e.is_within_group(GroupKind::AlterOpFamilyStmt) {
            // In ALTER statements, allow breaking after identifiers
            e.line(LineType::SoftOrSpace);
        } else {
            // Default: just add space
            e.space();
        }
    }
}

// Example: Conditional indentation based on parent context
impl ToTokens for OptionsList {
    fn to_tokens(&self, e: &mut EventEmitter) {
        match e.parent_group() {
            Some(GroupKind::AlterUserMappingStmt) |
            Some(GroupKind::AlterForeignDataWrapperStmt) => {
                // In ALTER statements, indent OPTIONS
                e.indent_start();
                e.line(LineType::SoftOrSpace);
                self.emit_options(e);
                e.indent_end();
            }
            _ => {
                // In other contexts, don't indent
                self.emit_options(e);
            }
        }
    }
}
```

#### Important Notes

- **Context helpers are read-only** - they don't modify the group stack
- **Check context before making formatting decisions** - don't assume a specific parent
- **Use sparingly** - most formatting should be consistent regardless of context
- **Prefer explicit context** over deep nesting checks

### Task Management and Progress Tracking

#### TodoWrite Tool - Critical for Progress Persistence

**The TodoWrite tool writes to an actual file** that persists between sessions. This is essential for:

```rust
// The TodoWrite tool creates/updates a real file containing your TODO list
// This file survives session interruptions and allows resuming work
TodoWrite {
    todos: [
        {
            content: "Fix ALTER statement formatting in test_formatter__alter_object_depends_stmt_0_60",
            status: "in_progress", 
            priority: "high",
            id: "1"
        },
        {
            content: "Review CREATE statement snapshots for proper indentation",
            status: "pending",
            priority: "medium", 
            id: "2"
        }
    ]
}
```

#### When to Use TodoWrite

**ALWAYS use TodoWrite for:**
- Starting a new formatting task/session
- Before beginning each major phase (discovery, review, fixes)
- After completing each test fix
- When discovering new formatting issues
- Before ending a session (to save progress)

**Example Usage Pattern:**
```rust
// At start of session
TodoWrite { todos: [/* initialize with planned work */] }

// During work
TodoWrite { todos: [/* update with current progress */] }

// When stopping work
TodoWrite { todos: [/* save current state for resumption */] }
```

### CRITICAL RULES

#### ❌ NEVER DO THESE:
1. **NEVER modify the renderer** - only change `nodes.rs`
2. **NEVER create random groups** - only use existing `GroupKind` variants
3. **NEVER use arbitrary group names** - stick to semantic groupings
4. **NEVER skip TodoWrite updates** - progress must be tracked persistently

#### ✅ ALWAYS DO THESE:
1. **Use existing group contexts** - leverage `GroupKind::SelectStmt`, `GroupKind::InsertStmt`, etc.
2. **Add breaks at clause boundaries** - before `WHERE`, `ORDER BY`, `GROUP BY`, etc.
3. **Use proper indentation** - wrap subordinate clauses with `indent_start()/indent_end()`
4. **Update TODO list frequently** - use TodoWrite to maintain persistent progress state

## Common Formatting Patterns

### ALTER Statements - FOCUS ON INDENTATION
```rust
// Pattern: Main statement + indented optional clauses
e.token(TokenKind::ALTER_KW);
e.space();
e.token(TokenKind::TABLE_KW);
e.space();
table_name.to_tokens(e);

// CRITICAL: Always indent optional clauses
if has_version_clause {
    e.indent_start();              // START indentation
    e.line(LineType::SoftOrSpace); // Break to new line
    e.token(TokenKind::VERSION_KW);
    e.space();
    version.to_tokens(e);
    e.indent_end();                // END indentation
}

if has_options_clause {
    e.indent_start();              // START indentation for OPTIONS
    e.line(LineType::SoftOrSpace); // Break to new line
    e.token(TokenKind::OPTIONS_KW);
    e.space();
    e.token(TokenKind::L_PAREN);
    // ... options content
    e.token(TokenKind::R_PAREN);
    e.indent_end();                // END indentation for OPTIONS
}
```

**Result:**
```sql
ALTER FOREIGN DATA WRAPPER fdw_name
  VERSION '1.2'        -- Indented optional clause
  OPTIONS (            -- Indented optional clause
    host 'example.com'
  );
```

### CREATE Statements
```rust
// Pattern: CREATE + main object + indented options
e.token(TokenKind::CREATE_KW);
e.space();
e.token(TokenKind::FUNCTION_KW);
// ...

if !parameters.is_empty() {
    e.space();
    e.token(TokenKind::L_PAREN);
    for (i, param) in parameters.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);  // Allow breaking between params
        }
        param.to_tokens(e);
    }
    e.token(TokenKind::R_PAREN);
}

if has_returns {
    e.line(LineType::SoftOrSpace);  // Break before RETURNS when needed
    e.token(TokenKind::RETURNS_KW);
    // ...
}
```

### MERGE Statements - HIERARCHICAL INDENTATION
```rust
// Pattern: Multi-level hierarchical indentation
e.token(TokenKind::MERGE_KW);
e.space();
e.token(TokenKind::INTO_KW);
// ...

e.line(LineType::SoftOrSpace);
e.token(TokenKind::USING_KW);
e.space();
source.to_tokens(e);
e.space();  // Keep ON on same line as USING when possible
e.token(TokenKind::ON_KW);
// ...

for when_clause in when_clauses {
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::WHEN_KW);
    // ...
    e.space();
    e.token(TokenKind::THEN_KW);
    
    // CRITICAL: Indent the action
    e.indent_start();                    // Level 1 indentation
    e.line(LineType::SoftOrSpace);
    
    match action_type {
        Update => {
            e.token(TokenKind::UPDATE_KW);
            e.space();
            e.token(TokenKind::SET_KW);
            e.indent_start();            // Level 2 indentation for SET items
            e.line(LineType::SoftOrSpace);
            // SET assignments here
            e.indent_end();              // End level 2
        }
        Insert => {
            e.token(TokenKind::INSERT_KW);
            // columns...
            e.indent_start();            // Level 2 indentation for VALUES
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::VALUES_KW);
            // values...
            e.indent_end();              // End level 2
        }
    }
    
    e.indent_end();                      // End level 1
}
```

**Result:**
```sql
MERGE INTO target_table t
USING source_table s ON t.id = s.id
WHEN MATCHED THEN
  UPDATE SET              -- Level 1 indent
    column = value        -- Level 2 indent
WHEN NOT MATCHED THEN
  INSERT (columns)        -- Level 1 indent
    VALUES (values);      -- Level 2 indent
```

## Testing and Validation

### Running Tests
```bash
# Run all pretty printer tests
cargo test -p pgt_pretty_print

# Run specific test with output
cargo test -p pgt_pretty_print test_formatter__merge_stmt_0_60 -- --show-output

# Run tests and check failure count
cargo test -p pgt_pretty_print 2>&1 | grep -E "FAILED.*failed" | tail -1
```

### Interpreting Test Results
Tests will fail with messages like:
```
Line exceeds max length of 60: CREATE FUNCTION add (a pg_catalog.int4, b pg_catalog.int4) RETURNS pg_catalog.int4 AS 'SELECT $1 + $2' LANGUAGE sql ;
```

This means you need to add line breaks to keep lines under 60 characters.

### Using pgFormatter for Inspiration
```bash
# Get formatting inspiration
pg_format crates/pgt_pretty_print/tests/data/create_function_stmt_0_60.sql

# This will show you how pgFormatter would format the statement
# Use this as inspiration, but adapt to our specific style
```

### Snapshot Updates
When tests fail due to formatting changes:
```bash
# Review and accept snapshot changes
cargo insta review

# Or run tests to see diffs
cargo insta test
```

## Step-by-Step Process

1. **Identify failing tests**:
   ```bash
   cargo test -p pgt_pretty_print | grep FAILED
   ```

2. **Examine a specific failure**:
   ```bash
   cargo test -p pgt_pretty_print test_formatter__create_function_stmt_0_60 -- --show-output
   ```

3. **Check PostgreSQL documentation** for the statement type

4. **Use pgFormatter for inspiration**:
   ```bash
   pg_format crates/pgt_pretty_print/tests/data/create_function_stmt_0_60.sql
   ```

5. **Find the ToTokens implementation** in `crates/pgt_pretty_print/src/nodes.rs`:
   ```bash
   grep -n "impl ToTokens.*CreateFunctionStmt" crates/pgt_pretty_print/src/nodes.rs
   ```

6. **Apply formatting improvements**:
   - Add `e.line(LineType::SoftOrSpace)` before major clauses
   - Add `e.indent_start()/e.indent_end()` around subordinate clauses
   - Use `e.space()` for required spaces

7. **Test the changes**:
   ```bash
   cargo test -p pgt_pretty_print test_formatter__create_function_stmt_0_60 -- --show-output
   ```

8. **Iterate until the formatting is correct**

## Examples of Good Formatting

### CREATE RULE (Documentation Style)
```sql
CREATE RULE notify_me AS ON UPDATE
  TO mytable
  DO NOTIFY mytable_updated;
```

### ALTER FOREIGN SERVER (Indented Options)
```sql
ALTER SERVER myserver
  VERSION '1.2'
  OPTIONS (host 'new.example.com');
```

### MERGE (Hierarchical Structure)
```sql
MERGE INTO target_table t
USING source_table s ON t.id = s.id
WHEN MATCHED THEN
  UPDATE SET
    value = s.value
WHEN NOT MATCHED THEN
  INSERT (id, value)
    VALUES (s.id, s.value);
```

## Common Statement Types to Fix

Focus on these patterns that commonly exceed line limits:
- ALTER statements with OPTIONS clauses
- CREATE FUNCTION with long parameter lists
- INSERT statements with ON CONFLICT
- MERGE statements (apply hierarchical formatting)
- CREATE TABLE with PARTITION BY
- Complex expressions and function calls

## Success Criteria

- All lines must be ≤ 60 characters (except JSON content)
- Formatting should follow PostgreSQL documentation patterns
- Indentation should show logical hierarchy
- Related clauses should stay together when possible
- Output should be readable and maintainable
- **JSON Content Rule**: Do not format JSON strings/objects - preserve them as-is even if they exceed line limits

Remember: The goal is readable, standards-compliant SQL formatting that respects line length constraints while maintaining semantic clarity.