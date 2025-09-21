# Task: Comprehensive Pretty Printer Formatting Review and Fix

## Overview

Perform a comprehensive review of ALL PostgreSQL pretty printer test snapshots to ensure proper SQL formatting that respects the 60-character line limit while following PostgreSQL documentation standards and the established formatting rules.

## Scope

This is a **complete formatting audit**, not just fixing current failures. The agent should:

1. **Enumerate all tests**: Get a complete list of all pretty printer tests
2. **Review each snapshot**: Examine the current formatting output for every test
3. **Apply formatting rules**: Check each snapshot against the formatting guide principles
4. **Fix violations**: Implement proper formatting for ANY test that doesn't follow the rules
5. **Validate improvements**: Ensure all changes improve readability and compliance

## Task Objectives

1. **Primary Goal**: Review and fix ALL test snapshots to follow formatting guide principles
2. **Line Limit**: Ensure no line exceeds 60 characters in any test output
3. **Standards Compliance**: Follow PostgreSQL documentation formatting patterns consistently
4. **Hierarchical Formatting**: Apply proper indentation to show SQL statement structure
5. **Comprehensive Coverage**: Don't just fix failures - improve ALL formatting

## Reference Documentation

**📚 REQUIRED READING**: See the comprehensive formatting guide at:
`/Users/psteinroe/Developer/postgres-language-server.git/feat/pretty-print/agentic/sql_formatter_improvements.md`

This guide contains:
- Formatting principles and indentation rules
- EventEmitter usage patterns
- Context-aware formatting helpers (`is_within_group()`, `parent_group()`)
- Common statement patterns (ALTER, CREATE, MERGE)
- Testing procedures and validation steps

## Comprehensive Review Process

### Phase 1: Test Discovery and Cataloging

1. **List all tests**:
   ```bash
   # Get complete list of pretty printer tests
   cargo test -p pgt_pretty_print --list | grep "test_formatter__" | sort
   ```

2. **Examine test snapshots**:
   ```bash
   # Find all snapshot files
   find crates/pgt_pretty_print/tests/snapshots -name "*.snap" | sort

   # Read snapshot content
   ls crates/pgt_pretty_print/tests/snapshots/tests__*.snap
   ```

3. **Review test input files**:
   ```bash
   # Find all test SQL input files
   find crates/pgt_pretty_print/tests/data -name "*.sql" | sort
   ```

### Phase 2: Systematic Snapshot Review

For EVERY test, examine the current snapshot and check:

#### Formatting Rule Compliance Checklist

**✅ Line Length Compliance**:
- [ ] No line exceeds 60 characters
- [ ] Long statements break at natural clause boundaries

**✅ Indentation Hierarchy**:
- [ ] Main statement keywords (CREATE, ALTER, MERGE) at base level
- [ ] Primary clauses (SELECT, FROM, WHERE) at base level
- [ ] Optional/subordinate clauses indented 1 level:
  - OPTIONS clauses in ALTER statements
  - AS clauses in CREATE statements
  - USING clauses in CREATE statements
  - TO/DO clauses in CREATE RULE
  - WHEN/THEN clauses in MERGE statements
- [ ] Sub-actions indented 2+ levels:
  - SET assignments in UPDATE
  - VALUES lists in INSERT
  - Column definitions in CREATE TYPE

**✅ PostgreSQL Documentation Patterns**:
- [ ] ALTER statements follow PostgreSQL manual formatting
- [ ] CREATE statements match official syntax examples
- [ ] MERGE statements use hierarchical structure
- [ ] Complex clauses broken logically

**✅ Readability and Structure**:
- [ ] Related elements stay together when possible
- [ ] Logical grouping of clauses
- [ ] Consistent spacing and breaks
- [ ] Clear visual hierarchy

### Implementation Requirements

- **File to modify**: `crates/pgt_pretty_print/src/nodes.rs` ONLY
- **Never modify**: `crates/pgt_pretty_print/src/renderer.rs`
- **Never create**: Random groups - use existing `GroupKind` variants only

## Success Criteria

✅ **All tests pass**: `cargo test -p pgt_pretty_print`
✅ **No line length violations**: All lines ≤ 60 characters
✅ **Proper indentation**: Subordinate clauses indented under main statements
✅ **PostgreSQL compliance**: Formatting matches official documentation patterns

## State Management and TODO List

**CRITICAL**: The agent MUST maintain both a comprehensive TODO list using the TodoWrite tool AND write state to a `STATE.md` file to track progress through ALL tests. This allows stopping and resuming the agentic loop without losing progress.

### State File Requirements

The agent MUST create and maintain a `STATE.md` file that contains:
- Current phase of work (Discovery, Review, Fixes, Validation)
- Last completed test or group
- Summary of issues found and fixed
- Current TODO list snapshot
- Any blocking issues or notes for resumption

### TODO List Structure

The agent should create and maintain a TODO list with the following categories:

```
Phase 1: Discovery
- [ ] Get complete list of all pretty printer tests
- [ ] Catalog all snapshot files
- [ ] Count total tests to review

Phase 2: Test Review (Systematic)
- [ ] Review ALTER statement tests (Group A: tests 1-50)
- [ ] Review CREATE statement tests (Group B: tests 51-100)
- [ ] Review SELECT/DML statement tests (Group C: tests 101-200)
- [ ] Review MERGE statement tests (Group D: tests 201-250)
- [ ] Review complex expression tests (Group E: tests 251-300)
- [ ] Review remaining statement tests (Group F: tests 301+)

Phase 3: Formatting Fixes
- [ ] Fix ALTER statement formatting issues
- [ ] Fix CREATE statement formatting issues
- [ ] Fix DML statement formatting issues
- [ ] Fix complex expression formatting issues
- [ ] Fix line length violations

Phase 4: Validation
- [ ] Run complete test suite
- [ ] Verify zero line length violations
- [ ] Check all snapshots for compliance
- [ ] Validate no regressions
```

### TODO List Updates

The agent MUST:
1. **Initialize TODO list** at the beginning of the task
2. **Update progress regularly** - mark items as in_progress/completed
3. **Add specific test names** as they are reviewed/fixed
4. **Track discovered issues** by adding new todos for found problems
5. **Maintain state persistence** so the task can be resumed later
6. **Update STATE.md file** after each major milestone or group completion
7. **Write current progress** to STATE.md before any long-running operations

### Example TODO Workflow

```
Initial State:
[1. [pending] Get complete list of all pretty printer tests (high)]
[2. [pending] Review ALTER statement test snapshots (high)]
...

After Discovery:
[1. [completed] Get complete list of all pretty printer tests (high)]
[2. [in_progress] Review ALTER statement test snapshots (high)]
[3. [pending] Fix alter_object_depends_stmt_0_60 line length (medium)]
[4. [pending] Fix alter_op_family_stmt_0_60 indentation (medium)]
...

After Each Fix:
[3. [completed] Fix alter_object_depends_stmt_0_60 line length (medium)]
[4. [in_progress] Fix alter_op_family_stmt_0_60 indentation (medium)]
...
```

## Implementation Strategy

### Step 1: Complete Test Discovery
```bash
# Get ALL pretty printer tests (not just failures)
cargo test -p pgt_pretty_print --list | grep "test_formatter__" > all_tests.txt

# Count total tests
wc -l all_tests.txt

# Get current snapshot files
find crates/pgt_pretty_print/tests/snapshots -name "tests__*.snap" | sort > all_snapshots.txt
```

### Step 2: Systematic Review Process
**IMPORTANT**: Update the TODO list after reviewing each group of tests.

For each test in the complete list:

1. **Update TODO list**: Mark current test group as "in_progress"

2. **Read current snapshot**:
   ```bash
   # View current formatting output
   cat crates/pgt_pretty_print/tests/snapshots/tests__[test_name].snap
   ```

3. **Check against formatting rules**:
   - Line length (≤60 characters)
   - Proper indentation hierarchy
   - PostgreSQL documentation compliance
   - Logical clause breaking

4. **Add specific issues to TODO list**:
   ```
   # If issues found, add to TODO list immediately:
   [X. [pending] Fix test_formatter__create_function_stmt_0_60 - line too long (medium)]
   [Y. [pending] Fix test_formatter__alter_table_stmt_0_60 - missing indentation (medium)]
   ```

5. **Run individual test**:
   ```bash
   cargo test -p pgt_pretty_print test_formatter__[test_name] -- --show-output
   ```

6. **Update TODO list**: Mark test group as "completed" when all tests in group are reviewed

### Step 3: Apply Systematic Improvements
Work through ALL tests systematically, not just failures:

1. **Update TODO list**: Mark each fix as "in_progress" when starting
2. **Group by statement type**: ALTER, CREATE, SELECT, INSERT, etc.
3. **Apply consistent patterns**: Use formatting guide rules uniformly
4. **Test each change**: Validate improvement before moving to next test
5. **Update TODO list**: Mark each fix as "completed" immediately after testing
6. **Add new issues**: If fixing one test reveals issues in related tests, add them to TODO list

### Step 4: Comprehensive Validation
```bash
# Run all tests to ensure no regressions
cargo test -p pgt_pretty_print

# Check for any remaining line length violations
cargo test -p pgt_pretty_print 2>&1 | grep "Line exceeds max length"

# Verify final count
cargo test -p pgt_pretty_print 2>&1 | grep "test result:"
```

### Step 5: Snapshot Management
```bash
# Review ALL snapshot changes (not just failures)
cargo insta review

# Run comprehensive test to validate all improvements
cargo insta test

# Accept all valid formatting improvements
cargo insta accept
```

## Key Formatting Principles (Summary)

1. **Indentation Hierarchy**:
   - Main keywords (CREATE, ALTER) → Base level
   - Primary clauses (SELECT, FROM) → Base level
   - Optional clauses (OPTIONS, AS, USING) → Indented 1 level
   - Sub-actions (SET assignments) → Indented 2 levels

2. **Line Breaking Strategy**:
   - Use `e.line(LineType::SoftOrSpace)` before major clauses
   - Use `e.indent_start()` / `e.indent_end()` around subordinate clauses
   - Apply context-aware formatting with `e.is_within_group()` when needed


3. **Testing Validation**:
   - PostgreSQL documentation patterns
   - pgFormatter inspiration: `pg_format path/to/file.sql`
   - Line length compliance

## Expected Outcome

After completing this comprehensive review:
- **All tests pass**: Zero test failures across all 376+ tests
- **Zero line length violations**: No line exceeds 60 characters in any snapshot
- **Consistent formatting**: All statements follow the formatting guide principles uniformly
- **Improved readability**: Enhanced hierarchical indentation across all SQL statement types
- **PostgreSQL compliance**: All formatting matches official documentation patterns
- **Visual consistency**: Uniform application of indentation and line breaking rules

## Completion Signal Requirements

**CRITICAL**: When the task is fully complete, the agent MUST output this exact phrase:

`TASK COMPLETE`

This allows automated scripts to detect task completion reliably.

## Deliverables

1. **Complete test list**: Documentation of all reviewed tests
2. **Formatting improvements**: Enhanced snapshots for ALL tests that needed improvement
3. **Zero regressions**: All previously passing tests continue to pass
4. **Comprehensive coverage**: Every test reviewed against the formatting checklist
5. **Clean test suite**: No line length violations or formatting inconsistencies

## Progress Tracking Requirements

**The agent MUST use the TodoWrite tool throughout the entire process to:**

1. **Maintain persistent state**: TODO list survives session interruptions
2. **Track detailed progress**: Specific test names and issues identified
3. **Enable resumption**: Can pick up exactly where left off
4. **Provide visibility**: User can see current progress and remaining work
5. **Prevent loss**: No work is lost if the session is interrupted

### Critical TODO List Rules

- **Always use TodoWrite** before and after major steps
- **Be specific**: Include test names, not just generic descriptions
- **Update frequently**: Mark progress on individual tests, not just groups
- **Track issues**: Add newly discovered problems to the list immediately
- **Maintain state**: TODO list is the authoritative source of current progress
- **Update STATE.md**: Write persistent state to file after each major milestone

## Important Notes

- **Comprehensive scope**: This is NOT just about fixing failures - review ALL tests
- **Proactive improvements**: Fix formatting issues even in currently passing tests
- **Consistent application**: Apply formatting rules uniformly across all statement types
- **Quality focus**: Prioritize readability and PostgreSQL documentation compliance
- **Systematic approach**: Work through tests methodically to ensure complete coverage
- **Validation**: Test each change to prevent regressions while improving formatting
- ****TODO LIST CRITICAL**: Use TodoWrite tool religiously to maintain progress state
