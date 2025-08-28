use anyhow::Result;
use std::fs;
use std::io::Write;
use std::process::Command;
use xtask::project_root;

use crate::claude_session::{AgenticState, ClaudeSession};

struct Logger {
    file: std::fs::File,
}

impl Logger {
    fn new() -> Result<Self> {
        let log_path = project_root().join("xtask/agentic/agent.log");
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

pub struct PrettyPrintGenerator {
    logger: Logger,
    claude_session: ClaudeSession,
    state: AgenticState,
}

impl PrettyPrintGenerator {
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
        self.log("Starting agentic pretty print implementation generator...");

        'outer_loop: loop {
            // Step 1: Pick next node
            let node = match self.pick_next_node() {
                Ok(n) => n,
                Err(_) => {
                    self.log("All nodes have been processed!");
                    break;
                }
            };

            self.log(&format!("\n=== Processing node: {} ===", node));
            self.state.current_node = node.clone();
            self.state.save()?;

            // Step 2: Get node structure (cached if available)
            self.log("Step 2: Analyzing node structure...");
            let node_info = self.get_cached_node_structure(&node)?;
            self.log(&format!("Node structure:\n{}", node_info));

            // Step 3: Generate test examples
            self.log("\nStep 3: Generating SQL examples...");
            let (mut filename, mut examples) = self.generate_test_examples(&node, &node_info)?;
            self.log(&format!("Generated: {} with SQL: {}", filename, examples));

            // Track iteration count for this node
            let iteration = self
                .state
                .in_progress_nodes
                .get(&node)
                .cloned()
                .unwrap_or(0);
            self.log(&format!("Implementation iteration: {}", iteration + 1));

            let mut retry_count = 0;
            let max_retries = 3;

            loop {
                // Step 4: Validate examples with AST analysis
                self.log("\nStep 4: Validating examples with AST analysis...");
                if !self.validate_with_ast_analysis(&examples, &node, &filename)? {
                    if retry_count >= max_retries {
                        self.log(&format!(
                            "❌ STOPPING: Failed to generate valid examples after {} retries for {}.",
                            max_retries, node
                        ));
                        self.log(
                            "This indicates a problem with the SQL generation or validation logic.",
                        );
                        self.state
                            .errors
                            .push(format!("{}: Failed to generate valid examples", node));
                        self.state.save()?;
                        return Err(anyhow::anyhow!(
                            "Failed to validate SQL examples for {}",
                            node
                        ));
                    }
                    self.log("Validation failed. Regenerating examples...");
                    let (new_filename, new_examples) =
                        self.generate_test_examples(&node, &node_info)?;
                    filename = new_filename;
                    examples = new_examples;
                    retry_count += 1;
                    continue;
                }

                // Step 5: Implement ToTokens and update nodes.rs
                self.log("\nStep 5: Implementing ToTokens trait and updating nodes.rs...");
                let nodes_file = project_root().join("crates/pgt_pretty_print/src/nodes.rs");

                // Save current file content for rollback if needed
                let existing_content = fs::read_to_string(&nodes_file).unwrap_or_default();

                self.implement_to_tokens_and_update_file(
                    &node,
                    &node_info,
                    &examples,
                    iteration,
                    &nodes_file,
                )?;
                self.log("Implementation written to nodes.rs and Node match updated");

                // Step 6: Check compilation first
                self.log("\nStep 6: Checking compilation...");
                let compilation_result = self.check_compilation_with_errors()?;
                if !compilation_result.0 {
                    self.log("Compilation failed. Implementation has syntax errors.");

                    // For compilation errors, always ask Claude to fix them
                    self.log("Asking Claude to fix compilation errors...");
                    self.fix_compilation_errors_directly(
                        &node,
                        &nodes_file,
                        &compilation_result.1,
                    )?;
                    self.log("Applied Claude's compilation fixes");

                    retry_count += 1;
                    if retry_count >= max_retries {
                        self.log("Max retries reached for compilation fixes");
                        self.state.errors.push(format!(
                            "{}: Compilation failed after {} attempts (iteration {})",
                            node,
                            max_retries,
                            iteration + 1
                        ));
                        break;
                    }
                    continue;
                }

                // Step 7: Run formatter tests
                self.log("\nStep 7: Running formatter tests...");
                let (test_success, test_errors) = self.run_formatter_tests(&node, &filename)?;
                if !test_success {
                    self.log(&format!("Tests failed. Error output:\n{}", test_errors));

                    // Check if this is a missing node error or an implementation issue
                    let missing_nodes =
                        self.extract_missing_node_types_with_claude(&test_errors)?;
                    if !missing_nodes.is_empty() {
                        self.log(&format!(
                            "Found missing node implementations: {:?}",
                            missing_nodes
                        ));

                        // Add these missing nodes to the front of our processing queue
                        for missing_node in &missing_nodes {
                            if !self.state.completed_nodes.contains(missing_node)
                                && !self.state.in_progress_nodes.contains_key(missing_node)
                            {
                                self.log(&format!(
                                    "Prioritizing {} for next implementation",
                                    missing_node
                                ));
                                self.state.in_progress_nodes.insert(missing_node.clone(), 0);
                            }
                        }
                        self.state.save()?;
                    } else {
                        // This is an implementation issue, not missing nodes
                        self.log(
                            "No missing nodes found. This appears to be an implementation issue.",
                        );
                        self.log("The current implementation may need to be improved rather than adding new nodes.");

                        // Ask Claude for guidance on the implementation issue
                        self.log("Getting Claude's analysis of the implementation issue...");
                        let analysis = self.analyze_implementation_issue(&node, &test_errors)?;
                        self.log(&format!("Claude's analysis: {}", analysis));

                        // Ask Claude to fix the implementation based on the test failure
                        self.log("Asking Claude to fix the ToTokens implementation based on test failure...");
                        self.fix_implementation_from_test_failure(
                            &node,
                            &nodes_file,
                            &test_errors,
                            &analysis,
                        )?;
                        self.log("Applied Claude's implementation fixes based on test failure");
                    }

                    if retry_count >= max_retries {
                        self.log(&format!(
                            "Tests failed after {} retries. Asking Claude for decision...",
                            max_retries
                        ));

                        // Let Claude decide whether to iterate or move on
                        let should_iterate = self.should_iterate_or_move_on(
                            &node,
                            &test_errors,
                            iteration,
                            retry_count,
                        )?;

                        if should_iterate {
                            self.log(&format!(
                                "Claude decided to try another iteration for {} (iteration {})",
                                node,
                                iteration + 1
                            ));
                            self.state
                                .in_progress_nodes
                                .insert(node.clone(), iteration + 1);
                            fs::write(&nodes_file, existing_content)?;
                            self.state.save()?;
                            continue 'outer_loop;
                        } else {
                            self.log(&format!(
                                "Claude decided to move to next node after {} failed attempts",
                                max_retries
                            ));
                            self.state.errors.push(format!(
                                "{}: Tests failed after {} attempts (iteration {})",
                                node,
                                max_retries,
                                iteration + 1
                            ));
                            break;
                        }
                    }

                    self.log(&format!(
                        "Retrying implementation... (attempt {} of {})",
                        retry_count + 1,
                        max_retries
                    ));

                    // Remove the failed implementation from nodes.rs
                    fs::write(&nodes_file, existing_content)?;

                    retry_count += 1;
                    continue;
                }

                // Step 8: Verify coverage
                self.log("\nStep 8: Verifying property coverage...");
                if !self.verify_node_coverage_from_file(&node, &nodes_file, &node_info)? {
                    self.log("Warning: Not all properties are covered in the implementation!");

                    // Let Claude decide whether to improve coverage or move on
                    self.log("Asking Claude whether to improve coverage...");
                    let should_iterate = self.should_improve_coverage_from_file(
                        &node,
                        &nodes_file,
                        &node_info,
                        iteration,
                    )?;

                    if should_iterate {
                        self.log(&format!(
                            "Claude decided to iterate to improve coverage for {}",
                            node
                        ));
                        self.state
                            .in_progress_nodes
                            .insert(node.clone(), iteration + 1);
                        self.state.save()?;
                        continue 'outer_loop;
                    } else {
                        self.log("Claude decided current coverage is sufficient for now");
                    }
                }

                // Success!
                self.log(&format!(
                    "\n✓ Successfully implemented {} (iteration {})",
                    node,
                    iteration + 1
                ));
                self.state.completed_nodes.push(node.clone());
                self.state.in_progress_nodes.remove(&node); // Remove from in-progress
                self.state.save()?;
                break;
            }

            // Auto-continue to next node
            self.log("\nAuto-continuing to next node...");
        }

        self.print_summary();
        Ok(())
    }

    fn print_summary(&mut self) {
        self.log("\n=== Summary ===");
        self.log(&format!(
            "Completed nodes: {}",
            self.state.completed_nodes.len()
        ));
        self.log(&format!(
            "In-progress nodes: {}",
            self.state.in_progress_nodes.len()
        ));
        self.log(&format!("Errors: {}", self.state.errors.len()));
        self.log(&format!(
            "Cached node structures: {}",
            self.state.node_structure_cache.len()
        ));
        self.log(&format!(
            "Cached match statement updates: {}",
            self.state.node_match_updated.len()
        ));

        if !self.state.in_progress_nodes.is_empty() {
            self.log("\nNodes in progress (with iteration counts):");
            let progress_nodes: Vec<_> = self
                .state
                .in_progress_nodes
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            for (node, iteration) in progress_nodes {
                self.log(&format!("  - {} (iteration {})", node, iteration));
            }
        }

        if !self.state.errors.is_empty() {
            self.log("\nNodes with errors:");
            let errors: Vec<_> = self.state.errors.clone();
            for error in errors {
                self.log(&format!("  - {}", error));
            }
        }
    }
}

impl PrettyPrintGenerator {
    fn pick_next_node(&self) -> Result<String> {
        // Read the AST nodes file from the xtask agentic crate
        let ast_nodes_path = project_root().join("xtask/agentic/ast_nodes.txt");
        let ast_nodes = fs::read_to_string(ast_nodes_path)?;
        let nodes: Vec<&str> = ast_nodes.lines().collect();

        // Strategy:
        // 1. First implement any in-progress nodes (nodes we started but haven't finished)
        // 2. Then Stmt nodes (as they're the top-level constructs)
        // 3. Then all other nodes

        // First pass: Look for in-progress nodes that aren't completed
        for node in nodes.iter() {
            let node_str = node.to_string();
            if self.state.in_progress_nodes.contains_key(&node_str)
                && !self.state.completed_nodes.contains(&node_str)
            {
                return Ok(node_str); // Continue with in-progress work
            }
        }

        // Second pass: Look for incomplete Stmt nodes
        for node in nodes.iter() {
            let node_str = node.to_string();
            if !node_str.ends_with("Stmt") {
                continue; // Skip non-Stmt nodes in this pass
            }

            if self.state.completed_nodes.contains(&node_str) {
                continue; // Already completed
            }

            return Ok(node_str);
        }

        // Third pass: Look for any incomplete node
        for node in nodes.iter() {
            let node_str = node.to_string();
            if self.state.completed_nodes.contains(&node_str) {
                continue; // Already completed
            }

            return Ok(node_str);
        }

        Err(anyhow::anyhow!("All nodes have been processed"))
    }

    fn get_cached_node_structure(&mut self, node: &str) -> Result<String> {
        // Check cache first
        if let Some(cached) = self.state.node_structure_cache.get(node).cloned() {
            self.log(&format!("✓ Using cached node structure for {}", node));
            return Ok(cached);
        }

        // Not cached, analyze with Claude
        self.log(&format!(
            "Analyzing {} structure with Claude (will cache for future use)",
            node
        ));
        let node_info = self.check_node_data(node)?;

        // Cache the result
        self.state
            .node_structure_cache
            .insert(node.to_string(), node_info.clone());
        self.state.save()?;

        Ok(node_info)
    }

    fn check_node_data(&mut self, node: &str) -> Result<String> {
        let prompt = format!(
            "Please analyze the struct {} in the file crates/pgt_query/src/protobuf.rs and list all its fields with their types. Format the response as a simple list.",
            node
        );

        self.claude_session.call_claude(&prompt, false)
    }

    fn generate_test_examples(&mut self, node: &str, node_info: &str) -> Result<(String, String)> {
        // Check if we have existing implementation and examples
        let existing_impl = self.get_existing_implementation(node)?;
        let existing_examples = self.get_existing_test_examples(node)?;

        // Calculate next test number
        let next_test_number = existing_examples.len();
        let base_filename = node.to_lowercase().replace("stmt", "_stmt");
        let filename = format!("{}_{}_{}", base_filename, next_test_number, 80);

        let guidance = if existing_impl.is_none() && existing_examples.is_empty() {
            // First time - generate minimal example
            "Generate the SIMPLEST possible PostgreSQL SQL statement that would create this AST node.\n\
            Use the ABSOLUTE MINIMUM - just the essential keywords and one simple example.\n\
            For example:\n\
            - INSERT should be just 'INSERT INTO t VALUES (1)'\n\
            - SELECT should be just 'SELECT 1'\n\
            - UPDATE should be just 'UPDATE t SET c = 1'"
        } else {
            // We have existing work - generate example that exercises unimplemented fields
            "Generate a PostgreSQL SQL statement that exercises the NEXT unimplemented feature.\n\
            Look at the existing implementation and examples to see what's already covered.\n\
            Generate SQL that would test a field or feature that's currently missing or marked with todo!().\n\
            Build incrementally - don't jump to the most complex example."
        };

        let context = if let Some(ref existing) = existing_impl {
            format!("EXISTING IMPLEMENTATION:\n{}\n\n", existing)
        } else {
            String::new()
        };

        let examples_context = if !existing_examples.is_empty() {
            format!(
                "EXISTING TEST EXAMPLES:\n{}\n\n",
                existing_examples.join("\n")
            )
        } else {
            String::new()
        };

        let test_dir = project_root().join("crates/pgt_pretty_print/tests/data");
        let final_filename = format!("{}.sql", filename);
        let test_file = test_dir.join(&final_filename);

        let prompt = format!(
            "{}\n\n\
            {}{}AST STRUCTURE:\n{}\n\
            \n\
            Generate a PostgreSQL SQL statement and write it to this file:\n\
            {}\n\
            \n\
            Use the Write tool to create the file with just the SQL statement (no extra text).\n\
            After writing, respond with just the SQL you wrote.",
            guidance,
            context,
            examples_context,
            node_info,
            test_file.display()
        );

        let sql = self.claude_session.call_claude(&prompt, false)?;
        let sql = sql.trim().to_string();

        self.log(&format!(
            "Claude wrote SQL to: {} with filename: {}",
            test_file.display(),
            final_filename
        ));

        Ok((final_filename, sql))
    }

    fn validate_with_ast_analysis(
        &mut self,
        sql: &str,
        expected_node: &str,
        filename: &str,
    ) -> Result<bool> {
        // Step 1: Test the specific file we created
        self.log("Step 1: Running file-specific validation...");
        if !self.validate_single_file(filename)? {
            return Ok(false);
        }

        // Step 2: Ask Claude to analyze if the SQL would create the expected node
        self.log(&format!(
            "Step 2: Getting Claude's analysis of SQL for {} node...",
            expected_node
        ));
        let prompt = format!(
            "I generated this SQL to create a {} AST node:\n{}\n\n\
            Does this SQL actually create a {} node when parsed by PostgreSQL? \
            Consider the SQL structure and what AST node type it would produce.\n\
            Reply with 'YES' if correct, or 'NO' with explanation and a corrected SQL example if wrong.",
            expected_node, sql, expected_node
        );

        let response = self.claude_session.call_claude(&prompt, false)?;
        let is_correct = response.trim().starts_with("YES");

        if !is_correct {
            self.log(&format!("Claude analysis: {}", response));
            return Ok(false);
        }

        self.log("✓ All validation steps passed!");
        Ok(true)
    }

    fn validate_single_file(&mut self, filename: &str) -> Result<bool> {
        // Use dir_test pattern - the test name is validate_test_data__ + filename without extension
        let test_name_suffix = filename.replace(".sql", "").replace("-", "_");
        let full_test_name = format!("validate_test_data__{}", test_name_suffix);

        self.log(&format!(
            "Running specific validation test: {}",
            full_test_name
        ));

        let output = Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("pgt_pretty_print")
            .arg(&full_test_name)
            .arg("--")
            .arg("--nocapture")
            .current_dir(project_root())
            .output()?;

        let success = output.status.success();

        if !success {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.log(&format!("Validation failed for {}:", filename));
            self.log(&format!("STDOUT:\n{}", stdout));
            self.log(&format!("STDERR:\n{}", stderr));
        } else {
            self.log(&format!("✓ Validation passed for {}", filename));
        }

        Ok(success)
    }

    fn implement_to_tokens_and_update_file(
        &mut self,
        node: &str,
        node_info: &str,
        examples: &str,
        iteration: u32,
        nodes_file: &std::path::Path,
    ) -> Result<()> {
        // Get existing implementation if it exists
        let existing_impl = self.get_existing_implementation(node)?;
        let trait_example = self.get_totokens_trait_example()?;

        let iteration_guidance = if iteration == 0 && existing_impl.is_none() {
            "This is the FIRST implementation. Focus on the ABSOLUTE MINIMUM to make progress:\n\
            - Implement only the essential keywords that make the SQL valid\n\
            - Use todo!() for complex fields that aren't immediately needed\n\
            - Goal: Get something that compiles and handles the simplest test case"
        } else if existing_impl.is_some() {
            "This is an ITERATIVE IMPROVEMENT of an existing implementation.\n\
            - Analyze the existing implementation and the AST structure\n\
            - Identify which fields are NOT yet implemented (marked with todo!() or missing)\n\
            - Add support for the NEXT most important field that would handle more test cases\n\
            - Keep all existing working code intact"
        } else {
            &format!(
                "This is iteration {} of the implementation. \
            Add more fields that are now unblocked because we've implemented their dependencies.",
                iteration + 1
            )
        };

        let context = if let Some(ref existing) = existing_impl {
            format!("EXISTING IMPLEMENTATION:\n{}\n\n", existing)
        } else {
            String::new()
        };

        // Check if we need to add the variant to the match statement
        let needs_match_update = !self.state.node_match_updated.contains(node);

        let prompt = format!(
            "Implement the ToTokens trait for {} in Rust and update nodes.rs file.\n\
            {}\n\n\
            FILE TO UPDATE: {}\n\n\
            TASKS:\n\
            1. First use Read tool to read the current nodes.rs file\n\
            {}2. Add the ToTokens implementation at the end of the file\n\
            3. Use Edit or MultiEdit to update the file\n\n\
            UNDERSTANDING THE TOTOKENS PATTERN:\n\
            The ToTokens trait converts AST nodes into formatting events for a pretty printer.\n\
            The EventEmitter records layout events that will later be rendered into formatted SQL.\n\n\
            EVENTEMIITTER API:\n\
            - e.token(TokenKind::KEYWORD) - Emit SQL keywords/symbols (INSERT_KW, COMMA, etc.)\n\
            - e.space() - Add a space\n\
            - e.line(LineType::SoftOrSpace) - Soft line break (becomes space if fits, newline if doesn't)\n\
            - e.line(LineType::Hard) - Force line break\n\
            - e.group_start(None, false) / e.group_end() - Logical formatting groups\n\
            - e.indent_start() / e.indent_end() - Control indentation\n\n\
            COMMON PATTERNS:\n\
            - Start with e.group_start(None, false) and end with e.group_end()\n\
            - Use e.space() between keywords: INSERT INTO becomes e.token(INSERT_KW); e.space(); e.token(INTO_KW)\n\
            - For optional fields: if let Some(field) = &self.field {{ field.to_tokens(e); }}\n\
            - For lists: iterate with commas between items\n\
            - Call .to_tokens(e) on child AST nodes to recursively format\n\n\
            TRAIT SIGNATURE EXAMPLE:\n{}\n\n\
            {}AST STRUCTURE:\n{}\n\n\
            SQL EXAMPLES TO HANDLE:\n{}\n\n\
            IMPLEMENTATION INSTRUCTIONS:\n\
            - Use the EXACT trait signature shown above\n\
            - {}\n\
            - NEVER add comments to the Rust code - no // comments, no /* */ comments\n\
            - Use existing TokenKind variants like INSERT_KW, INTO_KW, VALUES_KW, L_PAREN, R_PAREN, COMMA, etc.\n\
            - Add ONLY this file separator comment BEFORE the impl: // Implementation for {}\n\
            - The impl block and all code inside must have ZERO comments - absolutely no explanatory comments\n\
            - CRITICAL: Do not add comments like '// Handle the VALUES clause' or any explanations\n\n\
            After updating the file, just respond 'Done'.",
            node,
            iteration_guidance,
            nodes_file.display(),
            if needs_match_update {
                format!("2. Find the Node match statement and add this variant BEFORE the default case:\n   pgt_query::protobuf::node::Node::{}(node) => node.to_tokens(e),\n", node)
            } else {
                "".to_string()
            },
            trait_example,
            context,
            node_info,
            examples,
            if existing_impl.is_some() {
                "EXTEND the existing implementation, don't replace it completely"
            } else {
                "Start minimal"
            },
            node
        );

        self.claude_session.call_claude(&prompt, false)?;

        // Mark the node match as updated if needed
        if needs_match_update {
            self.state.node_match_updated.insert(node.to_string());
            self.state.save()?;
        }

        Ok(())
    }

    fn check_compilation_with_errors(&mut self) -> Result<(bool, String)> {
        self.log("Running cargo check on pgt_pretty_print...");

        let output = Command::new("cargo")
            .arg("check")
            .arg("-p")
            .arg("pgt_pretty_print")
            .current_dir(project_root())
            .output()?;

        let success = output.status.success();
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let full_output = format!("STDOUT:\\n{}\\nSTDERR:\\n{}", stdout, stderr);

        if !success {
            self.log("Compilation failed:");
            self.log(&full_output);
        } else {
            self.log("✓ Compilation successful");
        }

        Ok((success, full_output))
    }

    fn fix_compilation_errors_directly(
        &mut self,
        node: &str,
        nodes_file: &std::path::Path,
        compilation_errors: &str,
    ) -> Result<()> {
        let prompt = format!(
            "The ToTokens implementation for {} in {} has compilation errors.\\n\\n\
            COMPILATION ERRORS:\\n{}\\n\\n\
            Please:\\n\
            1. Use Read to examine the file\\n\
            2. Find the broken {} implementation\\n\
            3. Use Edit to fix the compilation errors\\n\\n\
            Fix requirements:\\n\
            - Keep the same overall structure and logic\\n\
            - Fix syntax errors, type errors, missing imports, etc.\\n\
            - Preserve the rest of the file exactly\\n\
            - NEVER add comments to the Rust code - no // comments, no /* */ comments\\n\\n\
            After fixing, just respond 'Fixed'.",
            node,
            nodes_file.display(),
            compilation_errors,
            node
        );

        self.claude_session.call_claude(&prompt, false)?;
        Ok(())
    }

    fn run_formatter_tests(&mut self, _node: &str, filename: &str) -> Result<(bool, String)> {
        // Run the specific formatter test for the node we just implemented
        let test_name = format!("test_formatter__{}", filename.replace(".sql", ""));

        self.log(&format!("Running test: {}", test_name));

        let output = Command::new("cargo")
            .arg("test")
            .arg("-p")
            .arg("pgt_pretty_print")
            .arg(&test_name)
            .arg("--")
            .arg("--nocapture")
            .current_dir(project_root())
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let success = output.status.success();
        let full_output = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);

        Ok((success, full_output))
    }

    fn extract_missing_node_types_with_claude(
        &mut self,
        error_output: &str,
    ) -> Result<Vec<String>> {
        let prompt = format!(
            "Analyze this Rust compiler/test error output and extract ONLY missing AST node type names:\n\n\
            {}\n\n\
            Look for errors that indicate missing ToTokens implementations, typically:\n\
            - \"Node type XYZ not implemented for to_tokens\"\n\
            - unimplemented!() panics mentioning specific node types\n\
            \n\
            Valid AST node names are like: AConst, List, Integer, SelectStmt, InsertStmt, String, etc.\n\
            \n\
            If you find missing node types, respond with just the node names, one per line:\n\
            NodeName1\n\
            NodeName2\n\
            \n\
            If this is NOT about missing node implementations (e.g., it's about incorrect output format, \
            AST differences, etc.), respond with:\n\
            IMPLEMENTATION_ISSUE\n\
            \n\
            Do not include explanations or other text.",
            error_output
        );

        let response = self.claude_session.call_claude(&prompt, false)?;
        let response = response.trim();

        if response == "IMPLEMENTATION_ISSUE" {
            self.log("Claude identified this as an implementation issue, not missing nodes");
            return Ok(Vec::new());
        }

        let missing_nodes: Vec<String> = response
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|s| s.to_string())
            .collect();

        self.log(&format!(
            "Claude extracted missing node types: {:?}",
            missing_nodes
        ));
        Ok(missing_nodes)
    }

    fn analyze_implementation_issue(&mut self, node: &str, error_output: &str) -> Result<String> {
        let prompt = format!(
            "The ToTokens implementation for {} is failing tests. The test works by:\n\
            1. Parse SQL to AST\n\
            2. Format AST back to SQL using ToTokens\n\
            3. Parse the formatted SQL to new AST\n\
            4. Compare original AST with new AST\n\n\
            TEST ERROR:\n{}\n\n\
            The error shows 'left' (original AST) vs 'right' (formatted->parsed AST). \
            Look at the differences between left and right ASTs.\n\n\
            This means the ToTokens implementation is NOT outputting certain fields properly, \
            causing the round-trip to lose data.\n\n\
            Analyze WHICH FIELDS are missing in the 'right' AST compared to 'left' AST. \
            These are the fields the ToTokens implementation needs to handle.\n\n\
            Focus on specific missing fields like values_lists, target_list, etc.",
            node, error_output
        );

        self.claude_session.call_claude(&prompt, false)
    }

    fn should_iterate_or_move_on(
        &mut self,
        node: &str,
        error_output: &str,
        iteration: u32,
        retry_count: u32,
    ) -> Result<bool> {
        let prompt = format!(
            "The ToTokens implementation for {} (iteration {}) failed after {} retries with this error:\n\
            {}\n\n\
            Should we try another iteration NOW, or move on and come back later?\n\
            Remember: ALL nodes eventually need FULL coverage. Moving on is just to make progress.\n\
            Consider:\n\
            - Does the error show we're blocked by missing node implementations?\n\
            - For complex nodes like SelectStmt, can we defer some features and implement dependent nodes first?\n\
            - Would implementing simpler nodes (like String, RangeVar) unblock this one?\n\
            \n\
            Example: SelectStmt can start with just 'SELECT 1' and add FROM/WHERE/etc later after implementing the nodes they depend on.\n\
            \n\
            Respond with ONLY 'ITERATE' to try another iteration now, or 'MOVE_ON' to implement dependencies first.",
            node, iteration + 1, retry_count, error_output
        );

        let response = self.claude_session.call_claude(&prompt, false)?;
        Ok(response.trim() == "ITERATE")
    }

    fn should_improve_coverage_from_file(
        &mut self,
        node: &str,
        nodes_file: &std::path::Path,
        node_info: &str,
        iteration: u32,
    ) -> Result<bool> {
        let prompt = format!(
            "The ToTokens implementation for {} (iteration {}) in {} is working but doesn't cover all properties.\n\
            Node structure:\n{}\n\n\
            Should we add more properties NOW, or move on and come back later?\n\n\
            Remember: ALL nodes eventually need FULL coverage. This is about sequencing.\n\n\
            Consider:\n\
            - Do we have enough to unblock other nodes? (e.g., SelectStmt with just SELECT/FROM might be enough to test RangeVar)\n\
            - Are the missing properties blocking progress on other nodes?\n\
            - Would implementing the dependent nodes first make this node easier to complete?\n\
            \n\
            Example: SelectStmt with basic SELECT/FROM is enough to move on, add WHERE/GROUP BY/etc after implementing their dependencies.\n\
            \n\
            Respond with ONLY 'ITERATE' to add more properties now, or 'MOVE_ON' to come back later.",
            node, iteration + 1, nodes_file.display(), node_info
        );

        let response = self.claude_session.call_claude(&prompt, false)?;
        Ok(response.trim() == "ITERATE")
    }

    fn verify_node_coverage_from_file(
        &mut self,
        node: &str,
        nodes_file: &std::path::Path,
        node_info: &str,
    ) -> Result<bool> {
        let prompt = format!(
            "Verify that the ToTokens implementation for {} in {} covers all fields.\n\n\
            Node structure:\n{}\n\n\
            Steps:\n\
            1. Use Read to find and examine the {} implementation\n\
            2. Check if all fields from the node structure are handled\n\n\
            Reply with only 'YES' if all fields are covered, or 'NO' followed by missing fields.",
            node,
            nodes_file.display(),
            node_info,
            node
        );

        let response = self.claude_session.call_claude(&prompt, false)?;
        Ok(response.trim().starts_with("YES"))
    }

    fn fix_implementation_from_test_failure(
        &mut self,
        node: &str,
        nodes_file: &std::path::Path,
        test_errors: &str,
        analysis: &str,
    ) -> Result<()> {
        let prompt = format!(
            "The ToTokens implementation for {} in {} is failing tests.\\n\\n\
            TEST FAILURE:\\n{}\\n\\n\
            ANALYSIS OF THE ISSUE:\\n{}\\n\\n\
            The test compares original AST with formatted->parsed AST. The 'right' AST is missing fields \
            that the 'left' AST has, meaning your ToTokens implementation isn't outputting those fields properly.\\n\\n\
            Please:\\n\
            1. Use Read to examine the current {} implementation in the file\\n\
            2. Identify which fields from the AST structure are not being handled\\n\
            3. Use Edit to improve the ToTokens implementation to handle the missing fields\\n\
            4. Focus on fields that appear in 'left' but are empty/missing in 'right'\\n\\n\
            For example, if left has `values_lists: [...]` but right has `values_lists: []`, \
            then you need to add code to handle the values_lists field in your ToTokens implementation.\\n\
            \\n\
            CRITICAL: NEVER add comments to the Rust code - no // comments, no /* */ comments\\n\\n\
            After fixing, just respond 'Fixed'.",
            node, nodes_file.display(), test_errors, analysis, node
        );

        self.claude_session.call_claude(&prompt, false)?;
        Ok(())
    }

    fn get_existing_implementation(&self, node: &str) -> Result<Option<String>> {
        let nodes_file = project_root().join("crates/pgt_pretty_print/src/nodes.rs");
        let content = fs::read_to_string(&nodes_file).unwrap_or_default();

        // Look for existing implementation of this node
        if let Some(start) =
            content.find(&format!("impl ToTokens for pgt_query::protobuf::{}", node))
        {
            if let Some(end) = content[start..].find("\n}\n") {
                let impl_block = &content[start..start + end + 2];
                return Ok(Some(impl_block.to_string()));
            }
        }

        Ok(None)
    }

    fn get_totokens_trait_example(&self) -> Result<String> {
        let nodes_file = project_root().join("crates/pgt_pretty_print/src/nodes.rs");
        let content = fs::read_to_string(&nodes_file).unwrap_or_default();

        // Extract an example implementation to show the correct signature
        if let Some(start) = content.find("impl ToTokens for pgt_query::protobuf::SelectStmt") {
            if let Some(end) = content[start..].find("}\n") {
                let example = &content[start..start + end + 1];
                return Ok(example.to_string());
            }
        }

        // Fallback if SelectStmt not found
        Ok("impl ToTokens for pgt_query::protobuf::YourNode {\n    fn to_tokens(&self, e: &mut EventEmitter) {\n        e.token(TokenKind::YOUR_KW);\n    }\n}".to_string())
    }

    fn get_existing_test_examples(&self, node: &str) -> Result<Vec<String>> {
        let test_dir = project_root().join("crates/pgt_pretty_print/tests/data");
        let mut examples = Vec::new();

        // Convert node name to expected filename pattern
        let base_pattern = node.to_lowercase().replace("stmt", "_stmt");

        if let Ok(entries) = fs::read_dir(&test_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Check if this file matches our naming pattern: base_pattern_N_80.sql
                        if filename.starts_with(&format!("{}_", base_pattern))
                            && filename.ends_with(".sql")
                        {
                            if let Ok(content) = fs::read_to_string(&path) {
                                examples.push(format!("{}: {}", filename, content.trim()));
                            }
                        }
                    }
                }
            }
        }

        // Sort examples by number to maintain order
        examples.sort();
        Ok(examples)
    }
}

pub fn run_pretty_print_generator() -> Result<()> {
    let mut generator = PrettyPrintGenerator::new()?;
    generator.run()
}
