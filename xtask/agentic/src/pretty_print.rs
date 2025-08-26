use anyhow::Result;
use std::fs;
use std::process::Command;
use xtask::project_root;

use crate::claude_session::{AgenticState, ClaudeSession};

pub fn run_pretty_print_generator() -> Result<()> {
    println!("Starting agentic pretty print implementation generator...");

    let mut state = AgenticState::load()?;
    let mut claude_session = ClaudeSession::new();

    'outer_loop: loop {
        // Step 1: Pick next node
        let node = match pick_next_node(&state) {
            Ok(n) => n,
            Err(_) => {
                println!("All nodes have been processed!");
                break;
            }
        };

        println!("\n=== Processing node: {} ===", node);
        state.current_node = node.clone();
        state.save()?;

        // Start a new conversation for each node to keep context clean
        println!("Step 2: Analyzing node structure...");
        let node_info = check_node_data(&mut claude_session, &node)?;
        println!("Node structure:\n{}", node_info);

        // Step 3: Generate test examples
        println!("\nStep 3: Generating SQL examples...");
        let examples_result = generate_test_examples(&mut claude_session, &node, &node_info)?;
        let parts: Vec<&str> = examples_result.split('|').collect();
        let (mut filename, mut examples) = if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("unknown.sql".to_string(), examples_result.clone())
        };
        println!("Generated: {} with SQL: {}", filename, examples);

        // Track iteration count for this node
        let iteration = state.in_progress_nodes.get(&node).cloned().unwrap_or(0);
        println!("Implementation iteration: {}", iteration + 1);

        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            // Step 4: Validate examples with AST analysis
            println!("\nStep 4: Validating examples with AST analysis...");
            if !validate_with_ast_analysis(&examples, &node, &mut claude_session, &filename)? {
                if retry_count >= max_retries {
                    println!(
                        "❌ STOPPING: Failed to generate valid examples after {} retries for {}.",
                        max_retries, node
                    );
                    println!(
                        "This indicates a problem with the SQL generation or validation logic."
                    );
                    state
                        .errors
                        .push(format!("{}: Failed to generate valid examples", node));
                    state.save()?;
                    return Err(anyhow::anyhow!(
                        "Failed to validate SQL examples for {}",
                        node
                    ));
                }
                println!("Validation failed. Regenerating examples...");
                let examples_result =
                    generate_test_examples(&mut claude_session, &node, &node_info)?;
                let parts: Vec<&str> = examples_result.split('|').collect();
                let (new_filename, new_examples) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    ("unknown.sql".to_string(), examples_result.clone())
                };
                filename = new_filename;
                examples = new_examples;
                retry_count += 1;
                continue;
            }

            // Step 5: Implement ToTokens
            println!("\nStep 5: Implementing ToTokens trait...");
            let implementation =
                implement_to_tokens(&mut claude_session, &node, &node_info, &examples, iteration)?;

            // Append implementation to nodes.rs file and add to Node match statement
            let nodes_file = project_root().join("crates/pgt_pretty_print/src/nodes.rs");
            let separator = format!("\n\n// Implementation for {}\n", node);
            let impl_content = format!("{}{}{}\n", separator, implementation, "");

            // Read existing content
            let existing_content = fs::read_to_string(&nodes_file).unwrap_or_default();

            // Add variant to Node match statement if it doesn't exist
            let updated_content = add_node_variant_to_match(&existing_content, &node)?;

            // Append the new implementation
            let final_content = format!("{}{}", updated_content, impl_content);
            fs::write(&nodes_file, final_content)?;
            println!(
                "Implementation appended to: {} and added to Node match",
                nodes_file.display()
            );

            // Step 6: Check compilation first
            println!("\nStep 6: Checking compilation...");
            if !check_compilation()? {
                println!("Compilation failed. Implementation has syntax errors.");
                if retry_count >= max_retries {
                    println!(
                        "Compilation failed after {} retries. Asking Claude for decision...",
                        max_retries
                    );

                    let should_iterate = should_iterate_or_move_on(
                        &mut claude_session,
                        &node,
                        "Compilation failed",
                        iteration,
                        retry_count,
                    )?;

                    if should_iterate {
                        println!(
                            "Claude decided to try another iteration for {} (iteration {})",
                            node,
                            iteration + 1
                        );
                        state.in_progress_nodes.insert(node.clone(), iteration + 1);
                        fs::write(&nodes_file, existing_content)?;
                        state.save()?;
                        continue 'outer_loop;
                    } else {
                        println!("Claude decided to move to next node after compilation failures");
                        state.errors.push(format!(
                            "{}: Compilation failed after {} attempts (iteration {})",
                            node,
                            max_retries,
                            iteration + 1
                        ));
                        break;
                    }
                }

                println!(
                    "Retrying implementation due to compilation errors... (attempt {} of {})",
                    retry_count + 1,
                    max_retries
                );
                fs::write(&nodes_file, existing_content)?;
                retry_count += 1;
                continue;
            }

            // Step 7: Run formatter tests
            println!("\nStep 7: Running formatter tests...");
            let (test_success, test_errors) = run_formatter_tests(&node, &filename)?;
            if !test_success {
                println!("Tests failed. Error output:\n{}", test_errors);

                // Extract missing nodes from the error and prioritize them
                let missing_nodes =
                    suggest_missing_implementations(&mut claude_session, &node, &test_errors)?;
                if !missing_nodes.is_empty() {
                    println!("Found missing node implementations: {:?}", missing_nodes);

                    // Add these missing nodes to the front of our processing queue
                    // by marking them as high-priority in our state
                    for missing_node in &missing_nodes {
                        if !state.completed_nodes.contains(missing_node)
                            && !state.in_progress_nodes.contains_key(missing_node)
                        {
                            println!("Prioritizing {} for next implementation", missing_node);
                            state.in_progress_nodes.insert(missing_node.clone(), 0);
                        }
                    }
                    state.save()?;
                }

                if retry_count >= max_retries {
                    println!(
                        "Tests failed after {} retries. Asking Claude for decision...",
                        max_retries
                    );

                    // Let Claude decide whether to iterate or move on
                    let should_iterate = should_iterate_or_move_on(
                        &mut claude_session,
                        &node,
                        &test_errors,
                        iteration,
                        retry_count,
                    )?;

                    if should_iterate {
                        println!(
                            "Claude decided to try another iteration for {} (iteration {})",
                            node,
                            iteration + 1
                        );
                        state.in_progress_nodes.insert(node.clone(), iteration + 1);
                        fs::write(&nodes_file, existing_content)?;
                        state.save()?;
                        continue 'outer_loop;
                    } else {
                        println!(
                            "Claude decided to move to next node after {} failed attempts",
                            max_retries
                        );
                        state.errors.push(format!(
                            "{}: Tests failed after {} attempts (iteration {})",
                            node,
                            max_retries,
                            iteration + 1
                        ));
                        break;
                    }
                }

                println!(
                    "Retrying implementation... (attempt {} of {})",
                    retry_count + 1,
                    max_retries
                );

                // Remove the failed implementation from nodes.rs
                fs::write(&nodes_file, existing_content)?;

                retry_count += 1;
                continue;
            }

            // Step 8: Verify coverage
            println!("\nStep 8: Verifying property coverage...");
            if !verify_node_coverage(&mut claude_session, &node, &implementation, &node_info)? {
                println!("Warning: Not all properties are covered in the implementation!");

                // Let Claude decide whether to improve coverage or move on
                println!("Asking Claude whether to improve coverage...");
                let should_iterate = should_improve_coverage(
                    &mut claude_session,
                    &node,
                    &implementation,
                    &node_info,
                    iteration,
                )?;

                if should_iterate {
                    println!("Claude decided to iterate to improve coverage for {}", node);
                    state.in_progress_nodes.insert(node.clone(), iteration + 1);
                    state.save()?;
                    continue 'outer_loop;
                } else {
                    println!("Claude decided current coverage is sufficient for now");
                }
            }

            // Success!
            println!(
                "\n✓ Successfully implemented {} (iteration {})",
                node,
                iteration + 1
            );
            state.completed_nodes.push(node.clone());
            state.in_progress_nodes.remove(&node); // Remove from in-progress
            state.save()?;
            break;
        }

        // Auto-continue to next node
        println!("\nAuto-continuing to next node...");
    }

    println!("\n=== Summary ===");
    println!("Completed nodes: {}", state.completed_nodes.len());
    println!("In-progress nodes: {}", state.in_progress_nodes.len());
    println!("Errors: {}", state.errors.len());

    if !state.in_progress_nodes.is_empty() {
        println!("\nNodes in progress (with iteration counts):");
        for (node, iteration) in &state.in_progress_nodes {
            println!("  - {} (iteration {})", node, iteration);
        }
    }

    if !state.errors.is_empty() {
        println!("\nNodes with errors:");
        for error in &state.errors {
            println!("  - {}", error);
        }
    }

    Ok(())
}

fn pick_next_node(state: &AgenticState) -> Result<String> {
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
        if state.in_progress_nodes.contains_key(&node_str)
            && !state.completed_nodes.contains(&node_str)
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

        if state.completed_nodes.contains(&node_str) {
            continue; // Already completed
        }

        return Ok(node_str);
    }

    // Third pass: Look for any incomplete node
    for node in nodes.iter() {
        let node_str = node.to_string();
        if state.completed_nodes.contains(&node_str) {
            continue; // Already completed
        }

        return Ok(node_str);
    }

    Err(anyhow::anyhow!("All nodes have been processed"))
}

fn suggest_missing_implementations(
    session: &mut ClaudeSession,
    node: &str,
    error_output: &str,
) -> Result<Vec<String>> {
    // First try to extract node names directly from "not implemented" errors
    let mut direct_suggestions = Vec::new();

    // Look for patterns like "Node type AConst(..." in the error
    for line in error_output.lines() {
        if line.contains("not implemented for to_tokens") {
            if let Some(start) = line.find("Node type ") {
                if let Some(end) = line[start + 10..].find('(') {
                    let node_name = &line[start + 10..start + 10 + end];
                    if !node_name.is_empty() && !direct_suggestions.contains(&node_name.to_string())
                    {
                        direct_suggestions.push(node_name.to_string());
                    }
                }
            }
        }
    }

    if !direct_suggestions.is_empty() {
        println!("Found missing nodes from error: {:?}", direct_suggestions);
        return Ok(direct_suggestions);
    }

    // Fallback to Claude analysis
    let prompt = format!(
        "The ToTokens implementation for {} failed with this error:\n\
        {}\n\n\
        Extract the specific AST node types that are missing implementations.\n\
        Look for errors like 'Node type XYZ not implemented for to_tokens'.\n\
        Respond with one node type name per line, just the type name.\n\
        If no missing nodes found, respond with 'NONE'",
        node, error_output
    );

    let response = session.call_claude(&prompt, false)?;
    let suggestions: Vec<String> = response
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && *line != "NONE")
        .map(|s| s.to_string())
        .collect();

    Ok(suggestions)
}

fn should_iterate_or_move_on(
    session: &mut ClaudeSession,
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

    let response = session.call_claude(&prompt, false)?;
    Ok(response.trim() == "ITERATE")
}

fn should_improve_coverage(
    session: &mut ClaudeSession,
    node: &str,
    implementation: &str,
    node_info: &str,
    iteration: u32,
) -> Result<bool> {
    let prompt = format!(
        "The ToTokens implementation for {} (iteration {}) is working but doesn't cover all properties.\n\
        Implementation:\n{}\n\
        Node structure:\n{}\n\n\
        Should we add more properties NOW, or move on and come back later?\n\
        Remember: ALL nodes eventually need FULL coverage. This is about sequencing.\n\
        Consider:\n\
        - Do we have enough to unblock other nodes? (e.g., SelectStmt with just SELECT/FROM might be enough to test RangeVar)\n\
        - Are the missing properties blocking progress on other nodes?\n\
        - Would implementing the dependent nodes first make this node easier to complete?\n\
        \n\
        Example: SelectStmt with basic SELECT/FROM is enough to move on, add WHERE/GROUP BY/etc after implementing their dependencies.\n\
        \n\
        Respond with ONLY 'ITERATE' to add more properties now, or 'MOVE_ON' to come back later.",
        node, iteration + 1, implementation, node_info
    );

    let response = session.call_claude(&prompt, false)?;
    Ok(response.trim() == "ITERATE")
}

fn check_node_data(session: &mut ClaudeSession, node: &str) -> Result<String> {
    let prompt = format!(
        "Please analyze the struct {} in the file crates/pgt_query/src/protobuf.rs and list all its fields with their types. Format the response as a simple list.",
        node
    );

    session.call_claude(&prompt, false)
}

fn generate_test_examples(
    session: &mut ClaudeSession,
    node: &str,
    node_info: &str,
) -> Result<String> {
    // Check if we have existing implementation and examples
    let existing_impl = get_existing_implementation(node)?;
    let existing_examples = get_existing_test_examples(node)?;

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

    let prompt = format!(
        "{}\n\n\
        {}{}AST STRUCTURE:\n{}\n\
        The test file will be named: {}.sql\n\
        \n\
        Generate only the SQL statement - no filename needed since it's predetermined.\n\
        Format your response as just:\n\
        SQL: your_sql_statement",
        guidance, context, examples_context, node_info, filename
    );

    let response = session.call_claude(&prompt, false)?;

    // Parse SQL from response
    let mut sql = String::new();

    for line in response.lines() {
        if let Some(statement) = line.strip_prefix("SQL: ") {
            sql = statement.trim().to_string();
            break;
        }
    }

    // Fallback if parsing failed
    if sql.is_empty() {
        sql = response.trim().to_string();
    }

    // Use the predetermined filename
    let final_filename = format!("{}.sql", filename);

    let test_dir = project_root().join("crates/pgt_pretty_print/tests/data");
    fs::create_dir_all(&test_dir)?;
    let test_file = test_dir.join(&final_filename);
    fs::write(&test_file, &sql)?;
    println!(
        "Wrote SQL to: {} with filename: {}",
        test_file.display(),
        final_filename
    );

    Ok(format!("{}|{}", final_filename, sql))
}

fn validate_single_file(filename: &str) -> Result<bool> {
    // Use dir_test pattern - the test name is validate_test_data__ + filename without extension
    let test_name_suffix = filename.replace(".sql", "").replace("-", "_");
    let full_test_name = format!("validate_test_data__{}", test_name_suffix);

    println!("Running specific validation test: {}", full_test_name);

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
        println!("Validation failed for {}:", filename);
        println!("STDOUT:\n{}", stdout);
        println!("STDERR:\n{}", stderr);
    } else {
        println!("✓ Validation passed for {}", filename);
    }

    Ok(success)
}

fn validate_with_ast_analysis(
    sql: &str,
    expected_node: &str,
    session: &mut ClaudeSession,
    filename: &str,
) -> Result<bool> {
    // Step 1: Test the specific file we created
    println!("Step 1: Running file-specific validation...");
    if !validate_single_file(filename)? {
        return Ok(false);
    }

    // Step 2: Ask Claude to analyze if the SQL would create the expected node
    println!(
        "Step 2: Getting Claude's analysis of SQL for {} node...",
        expected_node
    );
    let prompt = format!(
        "I generated this SQL to create a {} AST node:\n{}\n\n\
        Does this SQL actually create a {} node when parsed by PostgreSQL? \
        Consider the SQL structure and what AST node type it would produce.\n\
        Reply with 'YES' if correct, or 'NO' with explanation and a corrected SQL example if wrong.",
        expected_node, sql, expected_node
    );

    let response = session.call_claude(&prompt, false)?;
    let is_correct = response.trim().starts_with("YES");

    if !is_correct {
        println!("Claude analysis: {}", response);
        return Ok(false);
    }

    println!("✓ All validation steps passed!");
    Ok(true)
}

fn implement_to_tokens(
    session: &mut ClaudeSession,
    node: &str,
    node_info: &str,
    examples: &str,
    iteration: u32,
) -> Result<String> {
    // Get existing implementation if it exists
    let existing_impl = get_existing_implementation(node)?;
    let trait_example = get_totokens_trait_example()?;

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

    let prompt = format!(
        "Implement the ToTokens trait for {} in Rust.\n\
        {}\n\n\
        TRAIT SIGNATURE EXAMPLE (use this exact signature):\n{}\n\n\
        {}AST STRUCTURE:\n{}\n\n\
        SQL EXAMPLES TO HANDLE:\n{}\n\n\
        INSTRUCTIONS:\n\
        - Use the EXACT trait signature shown above\n\
        - {}
        - Generate ONLY the impl block, no imports or other code\n\
        - DO NOT use markdown code blocks - return plain Rust code\n\
        - Use existing TokenKind variants like INSERT_KW, INTO_KW, VALUES_KW, etc.",
        node,
        iteration_guidance,
        trait_example,
        context,
        node_info,
        examples,
        if existing_impl.is_some() {
            "EXTEND the existing implementation, don't replace it completely"
        } else {
            "Start minimal"
        }
    );

    session.call_claude(&prompt, false)
}

fn get_existing_implementation(node: &str) -> Result<Option<String>> {
    let nodes_file = project_root().join("crates/pgt_pretty_print/src/nodes.rs");
    let content = fs::read_to_string(&nodes_file).unwrap_or_default();

    // Look for existing implementation of this node
    if let Some(start) = content.find(&format!("impl ToTokens for pgt_query::protobuf::{}", node)) {
        if let Some(end) = content[start..].find("\n}\n") {
            let impl_block = &content[start..start + end + 2];
            return Ok(Some(impl_block.to_string()));
        }
    }

    Ok(None)
}

fn get_totokens_trait_example() -> Result<String> {
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

fn get_existing_test_examples(node: &str) -> Result<Vec<String>> {
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

fn add_node_variant_to_match(content: &str, node: &str) -> Result<String> {
    // Find the Node match statement and add our variant if it doesn't exist
    let variant_pattern = format!("pgt_query::protobuf::node::Node::{}(", node);

    // If the variant already exists, return content unchanged
    if content.contains(&variant_pattern) {
        return Ok(content.to_string());
    }

    // Find the match statement ending with "_ => {"
    if let Some(match_end) = content.find("            _ => {") {
        // Insert our new variant before the default case
        let before_default = &content[..match_end];
        let after_default = &content[match_end..];

        let new_variant = format!(
            "            pgt_query::protobuf::node::Node::{}(node) => node.to_tokens(e),\n            ",
            node
        );

        let updated = format!("{}{}{}", before_default, new_variant, after_default);
        println!("Added {} variant to Node match statement", node);
        return Ok(updated);
    }

    println!("Warning: Could not find Node match statement to update");
    Ok(content.to_string())
}

fn run_formatter_tests(_node: &str, filename: &str) -> Result<(bool, String)> {
    // Run the specific formatter test for the node we just implemented
    let test_name = format!("test_formatter__{}", filename.replace(".sql", ""));

    println!("Running test: {}", test_name);

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

fn check_compilation() -> Result<bool> {
    println!("Running cargo check on pgt_pretty_print...");

    let output = Command::new("cargo")
        .arg("check")
        .arg("-p")
        .arg("pgt_pretty_print")
        .current_dir(project_root())
        .output()?;

    let success = output.status.success();

    if !success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Compilation failed:");
        println!("STDOUT:\\n{}", stdout);
        println!("STDERR:\\n{}", stderr);
    } else {
        println!("✓ Compilation successful");
    }

    Ok(success)
}

fn verify_node_coverage(
    session: &mut ClaudeSession,
    node: &str,
    implementation: &str,
    node_info: &str,
) -> Result<bool> {
    let prompt = format!(
        "Verify that the following ToTokens implementation for {} covers all fields:\n\
        Implementation:\n{}\n\
        Node structure:\n{}\n\
        Reply with only 'YES' if all fields are covered, or 'NO' followed by missing fields.",
        node, implementation, node_info
    );

    let response = session.call_claude(&prompt, false)?;
    Ok(response.trim().starts_with("YES"))
}
