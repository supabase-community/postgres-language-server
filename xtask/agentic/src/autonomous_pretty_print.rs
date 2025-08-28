use anyhow::Result;
use std::fs;
use std::io::Write;
use xtask::project_root;

use crate::claude_session::{AgenticState, ClaudeSession};

struct Logger {
    file: std::fs::File,
}

impl Logger {
    fn new() -> Result<Self> {
        let log_path = project_root().join("xtask/agentic/autonomous_agent.log");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        Ok(Self { file })
    }

    fn log(&mut self, message: &str) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let log_line = format!("[{}] {}\n", timestamp, message);
        let _ = self.file.write_all(log_line.as_bytes());
        let _ = self.file.flush();
        // Also print to console
        print!("{}", log_line);
    }
}

pub struct AutonomousPrettyPrintGenerator {
    logger: Logger,
    claude_session: ClaudeSession,
    state: AgenticState,
}

impl AutonomousPrettyPrintGenerator {
    pub fn new() -> Result<Self> {
        let logger = Logger::new()?;
        let claude_session = ClaudeSession::new();
        let state = AgenticState::load()?;

        Ok(Self {
            logger,
            claude_session,
            state,
        })
    }

    fn log(&mut self, message: &str) {
        self.logger.log(message);
    }

    pub fn run(&mut self) -> Result<()> {
        self.log("Starting autonomous pretty print implementation generator...");

        loop {
            self.log("Starting new autonomous cycle...");

            let prompt = self.build_comprehensive_prompt()?;

            self.log("Sending comprehensive prompt to Claude...");
            let response = self.claude_session.call_claude(&prompt, false)?;

            self.log(&format!("Claude response: {}", response));

            // Check if Claude indicates it's done
            if response.contains("ALL_NODES_COMPLETED") {
                self.log("Claude reports all nodes are completed!");
                break;
            }

            // Small delay to prevent overwhelming
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        self.log("Autonomous pretty print generator completed!");
        Ok(())
    }

    fn build_comprehensive_prompt(&mut self) -> Result<String> {
        let ast_nodes_list = self.load_ast_nodes_list()?;
        let nodes_file_path = project_root().join("crates/pgt_pretty_print/src/nodes.rs");
        let test_data_dir = project_root().join("crates/pgt_pretty_print/tests/data");

        let prompt = format!(
            r#"You are an autonomous Rust code generator building a PostgreSQL pretty printer by implementing ToTokens traits for AST nodes.

## PROJECT CONTEXT:
This is a PostgreSQL pretty printer that takes parsed AST nodes and converts them back into well-formatted SQL code. Each AST node type needs a ToTokens implementation that defines how to reconstruct readable SQL from the parsed structure.

Your goal is to continuously implement ToTokens for all unimplemented nodes until the pretty printer is complete.

## YOUR INFINITE LOOP TASK:

1. **Read the current nodes.rs file** to see what's already implemented
2. **Pick the next unimplemented node** from the AST nodes list
3. **Analyze the node structure** in crates/pgt_query/src/protobuf.rs
4. **Generate a minimal SQL test case** and write it to tests/data/
5. **Validate the SQL example** by running: cargo test -p pgt_pretty_print --test tests validate_test_data__[filename]
6. **Implement the ToTokens trait** for the node
7. **Add the node to the match statement** if not already there
8. **Run compilation checks** with cargo check -p pgt_pretty_print
9. **Run the formatter tests** with cargo test -p pgt_pretty_print
10. **Validate AST round-trip** - Ensure the formatted SQL parses back to the same AST structure
11. **Check output snapshot** - Read the generated .snap file to verify the formatted output looks correct
12. **Fix any issues** that arise
13. **Repeat from step 1** until all nodes are implemented

## CRITICAL RULES:

- **NEVER EVER add comments to Rust code** - ZERO // comments, ZERO /* */ comments in implementations
- **Only add this separator**: // Implementation for NodeName
- **NEVER manually implement nested nodes** - Always call .to_tokens(e) on child nodes, never implement their logic
- **Use existing ToTokens implementations** - If a node already has ToTokens, call it, don't reimplement
- **Start with simplest possible implementations** - use todo!() for complex fields initially
- **Incremental improvement** - come back to nodes later to add more fields
- **When tests fail**, analyze the AST diff and fix the missing ToTokens fields
- **Prioritize Stmt nodes first** (SelectStmt, InsertStmt, etc.)

## FORBIDDEN PATTERNS:
- `// Handle XYZ formatting directly` - NEVER add explanatory comments
- `if let Some(node::Node::SomeType(inner)) = &node.node { /* manual implementation */ }` - NEVER manually implement child nodes
- Always use: `child_node.to_tokens(e)` instead of manual implementations

## FILE PATHS:
- Nodes to implement: {}
- Target file: {}
- Test directory: {}

## AST NODES TO IMPLEMENT:
{}

## TOOLS AVAILABLE:
- Read: Read any file
- Write: Create new files
- Edit/MultiEdit: Modify existing files
- Bash: Run commands like cargo check, cargo test
- Grep: Search for patterns in files

## VALIDATION PROCESS:
After creating a SQL file, validate it with:
```bash
cargo test -p pgt_pretty_print --test tests validate_test_data__[filename_without_sql]
```
Example: For insert_stmt_0_80.sql, run:
```bash
cargo test -p pgt_pretty_print --test tests validate_test_data__insert_stmt_0_80
```
This ensures the SQL parses correctly and produces the expected AST node type.

## OUTPUT VERIFICATION PROCESS:
After running formatter tests, check the quality by:
1. **Read the snapshot file**: `crates/pgt_pretty_print/tests/snapshots/tests__test_formatter__[filename].snap`
2. **Verify formatted output**: Check that the generated SQL looks properly formatted
3. **Validate round-trip**: Parse the formatted SQL and compare AST structure with original
4. **Look for common issues**: Missing whitespace, incorrect operator precedence, malformed syntax

Example: For insert_stmt_0_80.sql, check:
`crates/pgt_pretty_print/tests/snapshots/tests__test_formatter__insert_stmt_0_80.snap`

## RESPONSE FORMAT:
After each cycle, respond with just:
- "CONTINUING" to keep going
- "ALL_NODES_COMPLETED" when finished

## CORRECT IMPLEMENTATION PATTERN:
```rust
impl ToTokens for SomeNode {
    fn to_tokens(&self, e: &mut Elements) {
        // Implementation for SomeNode
        if let Some(ref child) = self.some_child {
            child.to_tokens(e);  // ✅ CORRECT - delegate to existing ToTokens
        }
        e.token(TokenKind::KEYWORD("SELECT".to_string()));
        // NO COMMENTS ANYWHERE ELSE
    }
}
```

## WRONG IMPLEMENTATION PATTERN (FORBIDDEN):
```rust
impl ToTokens for SomeNode {
    fn to_tokens(&self, e: &mut Elements) {
        // Implementation for SomeNode
        if let Some(ref child) = self.some_child {
            // Handle child formatting directly ❌ FORBIDDEN COMMENT
            if let Some(node::Node::ChildType(inner)) = &child.node {
                // ❌ FORBIDDEN - manual implementation of child
                e.token(TokenKind::IDENT(inner.name.clone()));
            }
        }
    }
}
```

## ACTION PLAN:
Start by reading nodes.rs to see current state, then begin the implementation loop. Work systematically through the nodes list, implementing ToTokens for each one with proper error handling and testing.

ALWAYS delegate to existing ToTokens implementations. NEVER add explanatory comments.

Continue this loop indefinitely until all nodes are implemented and all tests pass.
"#,
            "xtask/agentic/ast_nodes.txt",
            nodes_file_path.display(),
            test_data_dir.display(),
            ast_nodes_list
        );

        Ok(prompt)
    }

    fn load_ast_nodes_list(&self) -> Result<String> {
        let ast_nodes_path = project_root().join("xtask/agentic/ast_nodes.txt");
        fs::read_to_string(ast_nodes_path)
            .map_err(|e| anyhow::anyhow!("Failed to read AST nodes: {}", e))
    }
}

pub fn run_autonomous_pretty_print_generator() -> Result<()> {
    let mut generator = AutonomousPrettyPrintGenerator::new()?;
    generator.run()
}
