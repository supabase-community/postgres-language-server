use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use regex::Regex;
use std::env;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{BufRead, Cursor, Write};
use std::process::Command;

const OUTPUT_DIR: &str = "crates/pgt_pretty_print/tests/data/regression_suite";

fn find_project_root() -> Result<Utf8PathBuf> {
    // Start from the current directory and walk up until we find Cargo.toml
    let mut current_dir = Utf8PathBuf::try_from(env::current_dir()?)?;
    loop {
        if current_dir.join("Cargo.toml").exists() && current_dir.join("crates").exists() {
            return Ok(current_dir);
        }
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            bail!("Could not find project root");
        }
    }
}

pub fn download_regression_tests() -> Result<()> {
    let project_root = find_project_root()?;
    let target_dir = project_root.join(OUTPUT_DIR);

    if target_dir.exists() {
        println!("cleaning target directory: {:?}", target_dir);
        remove_dir_all(&target_dir)?;
    }

    create_dir_all(&target_dir)?;

    let urls = fetch_download_urls()?;
    let total_files = urls.len();

    for (index, url) in urls.iter().enumerate() {
        let filename = url.split('/').last().unwrap();
        if filename.contains("psql") {
            // skipping this for now, we don't support psql
            continue;
        }

        println!(
            "[{}/{}] Downloading {}... ",
            index + 1,
            total_files,
            filename
        );

        let output = Command::new("curl").args(["-s", url]).output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            bail!(anyhow::anyhow!(
                "Failed to download '{}': {}",
                url,
                error_msg
            ));
        }

        let cursor = Cursor::new(&output.stdout);

        if let Err(e) = preprocess_sql(cursor, &target_dir, filename) {
            bail!("Error: Failed to process file: {}", e);
        }
    }

    Ok(())
}

fn fetch_download_urls() -> Result<Vec<String>> {
    println!("Fetching SQL file URLs...");
    let output = Command::new("gh")
        .args([
            "api",
            "-H",
            "Accept: application/vnd.github+json",
            "/repos/postgres/postgres/contents/src/test/regress/sql",
        ])
        .output()?;

    if !output.status.success() {
        bail!(anyhow::anyhow!(
            "Failed to fetch SQL files: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_str = String::from_utf8(output.stdout)?;
    let files: Vec<serde_json::Value> = serde_json::from_str(&json_str)?;

    // extract download urls for sql files
    let urls: Vec<String> = files
        .into_iter()
        .filter(|file| {
            file["name"]
                .as_str()
                .map(|name| name.ends_with(".sql"))
                .unwrap_or(false)
        })
        .filter_map(|file| file["download_url"].as_str().map(String::from))
        .collect();

    if urls.is_empty() {
        bail!(anyhow::anyhow!("No SQL files found"));
    }

    Ok(urls)
}

fn parse_and_write_statement(
    statement: &str,
    base_path: &Utf8PathBuf,
    original_filename: &str,
    statement_counter: &mut usize,
    line_idx: Option<usize>,
) -> Result<()> {
    let trimmed = statement.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let parse_result = pgt_query::parse(trimmed);
    match parse_result {
        Ok(r) => {
            if r.root().is_none() {
                let line_info = line_idx
                    .map(|idx| format!("at line {}", idx + 1))
                    .unwrap_or_else(|| "".to_string());
                bail!(
                    "Parsed SQL statement {} has no root node:\nStatement:\n{}",
                    line_info,
                    trimmed
                );
            }
            // Generate filename for this statement
            let base_name = original_filename
                .strip_suffix(".sql")
                .unwrap_or(original_filename);
            let filename = format!("{}_stmt_{:03}_60.sql", base_name, *statement_counter);
            let filepath = base_path.join(filename);

            println!("Writing statement {} to {}", *statement_counter, filepath);

            let mut file = File::create(&filepath)?;
            writeln!(file, "{}", trimmed)?;

            *statement_counter += 1;
        }
        Err(e) => {
            let ignore_errors = [
                "Invalid statement: cannot use multiple ORDER BY clauses with WITHIN GROUP",
                "Invalid statement: column number must be in range from 1 to 32767",
                "Invalid statement: syntax error at or near \"ENFORCED\"",
                "Invalid statement: syntax error at or near \"NOT\"",
                "Invalid statement: syntax error at or near \"WITH\"",
                "Invalid statement: syntax error at or near \"enforced\"",
                "Invalid statement: syntax error at or near \"not\"",
                "Invalid statement: syntax error at or near \"NO\"",
                "Invalid statement: syntax error at or near \"COLLATE\"",
            ];

            if ignore_errors.iter().any(|msg| e.to_string().contains(msg)) {
                let line_info = line_idx
                    .map(|idx| format!(" at line {}", idx + 1))
                    .unwrap_or_else(|| "".to_string());
                println!(
                    "Ignoring parse error{}: {}\nStatement:\n{}",
                    line_info, e, trimmed
                );
            } else {
                let line_info = line_idx
                    .map(|idx| format!(" at line {}", idx + 1))
                    .unwrap_or_else(|| "".to_string());
                bail!(
                    "Failed to parse SQL statement{}: {}\nStatement:\n{}",
                    line_info,
                    e,
                    trimmed
                );
            }
        }
    }
    Ok(())
}

pub fn preprocess_sql<R: BufRead>(
    source: R,
    base_path: &Utf8PathBuf,
    original_filename: &str,
) -> Result<()> {
    let mut skipping_copy_block = false;

    let template_vars_regex =
        Regex::new(r#"^:'([^']+)'|^:"([^"]+)"|^:([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap();
    let dollar_quote_regex = Regex::new(r"\$([a-zA-Z]*)\$").unwrap();

    let mut current_statement = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_quote_tag = String::new();
    let mut in_single_quote = false;
    let mut escape_next = false;
    let mut in_multiline_comment = false;
    let mut statement_counter = 1;

    for (idx, line) in source.lines().enumerate() {
        let mut line = line?;

        // detect the start of the copy block
        if line.starts_with("COPY ") && line.to_lowercase().contains("from stdin") {
            skipping_copy_block = true;
            continue;
        }

        // detect the end of the copy block
        if skipping_copy_block && (line.starts_with("\\.") || line.is_empty()) {
            skipping_copy_block = false;
            continue;
        }

        // skip lines if inside a copy block
        if skipping_copy_block {
            continue;
        }

        // skip lines starting with a number followed by a space (data lines)
        if line.chars().next().map_or(false, |c| c.is_ascii_digit())
            && line
                .chars()
                .find(|c| !c.is_ascii_digit())
                .map_or(false, |c| c.is_whitespace())
        {
            continue;
        }

        if line.starts_with("--") {
            // skip comments
            continue;
        }

        if line.starts_with("\\") {
            // skip plpgsql commands (for now)
            continue;
        }

        // replace "\gset" with ";"
        if line.contains("\\gset") {
            line = line.replace("\\gset", ";");
        }

        // replace template variables
        let mut result = String::new();
        let mut i = 0;
        let bytes = line.as_bytes();
        let mut local_in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_array = false;

        while i < bytes.len() {
            let c = bytes[i] as char;

            // Handle quote state transitions
            match c {
                '\'' => {
                    result.push(c);
                    i += 1;
                    local_in_single_quote = !local_in_single_quote;
                    continue;
                }
                '"' => {
                    result.push(c);
                    i += 1;
                    in_double_quote = !in_double_quote;
                    continue;
                }
                '[' => {
                    result.push(c);
                    i += 1;
                    in_array = true;
                    continue;
                }
                ']' => {
                    result.push(c);
                    i += 1;
                    in_array = false;
                    continue;
                }
                ':' if !local_in_single_quote && !in_double_quote && !in_array => {
                    // Skip type casts (e.g., ::text)
                    if i + 1 < bytes.len() && bytes[i + 1] as char == ':' {
                        result.push_str("::");
                        i += 2;
                        continue;
                    }

                    if i + 2 < bytes.len() && bytes[i + 1] as char == '=' {
                        result.push_str(":=");
                        i += 2;
                        continue;
                    }

                    let remaining = &line[i..];
                    if let Some(caps) = template_vars_regex.captures(remaining) {
                        let full = caps.get(0).unwrap();

                        // Check which pattern matched to determine the quote style
                        if let Some(m) = caps.get(1) {
                            // :'string' format - keep as single quotes
                            let matched_var = &remaining[m.start()..m.end()];
                            println!("#{} Replacing template variable {}", idx, matched_var);
                            result.push('\'');
                            result.push_str(matched_var);
                            result.push('\'');
                        } else if let Some(m) = caps.get(2) {
                            // :"identifier" format - keep as double quotes
                            let matched_var = &remaining[m.start()..m.end()];
                            println!("#{} Replacing template variable {}", idx, matched_var);
                            result.push('"');
                            result.push_str(matched_var);
                            result.push('"');
                        } else if let Some(m) = caps.get(3) {
                            // :identifier format - convert to single quotes
                            let matched_var = &remaining[m.start()..m.end()];
                            println!("#{} Replacing template variable {}", idx, matched_var);
                            result.push('\'');
                            result.push_str(matched_var);
                            result.push('\'');
                        }

                        i += full.end();
                        continue;
                    }
                }
                _ => {}
            }

            result.push(c);
            i += 1;
        }

        // remove everything after -- in the line (inline comments)
        if let Some(pos) = result.find("--") {
            result.truncate(pos);
        }

        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        let mut line_buffer = String::new();

        while i < chars.len() {
            let ch = chars[i];

            if escape_next {
                escape_next = false;
                line_buffer.push(ch);
                i += 1;
                continue;
            }

            // Handle multiline comments
            if ch == '/' && !in_dollar_quote && !in_single_quote && !in_multiline_comment {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    // Start of multiline comment
                    in_multiline_comment = true;
                    line_buffer.push(ch);
                    line_buffer.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
            } else if ch == '*' && in_multiline_comment {
                if i + 1 < chars.len() && chars[i + 1] == '/' {
                    // End of multiline comment
                    in_multiline_comment = false;
                    line_buffer.push(ch);
                    line_buffer.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
            }

            if ch == '\'' && !in_dollar_quote && !in_multiline_comment {
                if in_single_quote {
                    if i + 1 < chars.len() && chars[i + 1] == '\'' {
                        line_buffer.push(ch);
                        line_buffer.push(chars[i + 1]);
                        i += 2;
                        continue;
                    } else {
                        in_single_quote = false;
                    }
                } else {
                    in_single_quote = true;
                }
                line_buffer.push(ch);
                i += 1;
                continue;
            }

            if ch == '\\' && in_single_quote {
                escape_next = true;
                line_buffer.push(ch);
                i += 1;
                continue;
            }

            if ch == '$' && !in_dollar_quote && !in_single_quote && !in_multiline_comment {
                if let Some(caps) = dollar_quote_regex.find_at(&result, i) {
                    let tag = caps.as_str();
                    in_dollar_quote = true;
                    dollar_quote_tag = tag.to_string();
                    for j in i..i + tag.len() {
                        if j < chars.len() {
                            line_buffer.push(chars[j]);
                        }
                    }
                    i += tag.len();
                    continue;
                }
            } else if ch == '$' && in_dollar_quote && !in_single_quote && !in_multiline_comment {
                let remaining = chars[i..].iter().collect::<String>();
                if remaining.starts_with(&dollar_quote_tag) {
                    in_dollar_quote = false;
                    for j in i..i + dollar_quote_tag.len() {
                        if j < chars.len() {
                            line_buffer.push(chars[j]);
                        }
                    }
                    i += dollar_quote_tag.len();
                    dollar_quote_tag.clear();
                    continue;
                }
            }

            line_buffer.push(ch);

            // check for statement termination (semicolon outside of quotes and comments)
            if ch == ';' && !in_dollar_quote && !in_single_quote && !in_multiline_comment {
                current_statement.push_str(&line_buffer);
                parse_and_write_statement(
                    &current_statement,
                    base_path,
                    original_filename,
                    &mut statement_counter,
                    Some(idx),
                )?;
                current_statement.clear();
                line_buffer.clear();
            }

            i += 1;
        }

        // add any remaining content from the line
        if !line_buffer.is_empty() {
            current_statement.push_str(&line_buffer);
            if !line_buffer.ends_with('\n') {
                current_statement.push('\n');
            }
        }
    }

    // Write any remaining statement that didn't end with a semicolon
    parse_and_write_statement(
        &current_statement,
        base_path,
        original_filename,
        &mut statement_counter,
        None,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_preprocess_sql(sql: &str) -> Result<String> {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Create a unique temporary directory for test files
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("test_sql_statements_{}", timestamp));
        let temp_dir_utf8 = Utf8PathBuf::try_from(temp_dir).unwrap();

        fs::create_dir_all(&temp_dir_utf8)?;

        let input = sql.as_bytes();
        let cursor = Cursor::new(input);
        preprocess_sql(cursor, &temp_dir_utf8, "test.sql")?;

        // Read all generated files and concatenate their contents
        let mut result = String::new();
        let mut entries: Vec<_> = fs::read_dir(&temp_dir_utf8)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("test_stmt_")
            })
            .collect();

        // Sort by filename to maintain order
        entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for entry in entries {
            let content = fs::read_to_string(entry.path())?;
            result.push_str(&content);
        }

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir_utf8); // Ignore cleanup errors

        Ok(result)
    }

    #[test]
    fn test_replacement() {
        let cases = [
            (
                "SELECT * FROM foo WHERE bar = :'foo' AND baz = :baz;",
                "SELECT * FROM foo WHERE bar = 'foo' AND baz = 'baz';",
            ),
            (
                "select array_dims('{1,2,3}'::dia);",
                "select array_dims('{1,2,3}'::dia);",
            ),
            (
                "SELECT to_char(now(), 'OF') as \"OF\", to_char(now(), 'TZH:TZM') as \"TZH:TZM\";",
                "SELECT to_char(now(), 'OF') as \"OF\", to_char(now(), 'TZH:TZM') as \"TZH:TZM\";",
            ),
            (
                "SELECT ('{{{1},{2},{3}},{{4},{5},{6}}}'::int[])[1][1:NULL][1];",
                "SELECT ('{{{1},{2},{3}},{{4},{5},{6}}}'::int[])[1][1:NULL][1];",
            ),
            (
                "SELECT JSON_OBJECT('foo': NULL::int FORMAT JSON);",
                "SELECT JSON_OBJECT('foo': NULL::int FORMAT JSON);",
            ),
            (
                "ALTER DATABASE :\"datname\" REFRESH COLLATION VERSION;",
                "ALTER DATABASE \"datname\" REFRESH COLLATION VERSION;",
            ),
        ];

        for (input, expected) in &cases {
            let result = test_preprocess_sql(input).unwrap();
            assert_eq!(result, format!("{}\n", *expected));
        }
    }

    #[test]
    fn test_dollar_quoted_strings() {
        let input = "CREATE FUNCTION test() RETURNS text AS $$SELECT 'test;';$$ LANGUAGE sql;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "CREATE FUNCTION test() RETURNS text AS $$SELECT 'test;';$$ LANGUAGE sql;\n"
        );

        let input = "CREATE FUNCTION test() RETURNS void AS $$\nBEGIN\n  RAISE NOTICE 'Hello;';\nEND;\n$$ LANGUAGE plpgsql;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(result, "CREATE FUNCTION test() RETURNS void AS $$\nBEGIN\n  RAISE NOTICE 'Hello;';\nEND;\n$$ LANGUAGE plpgsql;\n");

        let input =
            "CREATE FUNCTION test() RETURNS text AS $tag$SELECT 'test;';$tag$ LANGUAGE sql;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "CREATE FUNCTION test() RETURNS text AS $tag$SELECT 'test;';$tag$ LANGUAGE sql;\n"
        );
    }

    #[test]
    fn test_multiple_statements_on_one_line() {
        let input = "begin; alter table alterlock alter column f2 set statistics 150;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "begin;\nalter table alterlock alter column f2 set statistics 150;\n"
        );

        let input = "select 1; select 2; select 3;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(result, "select 1;\nselect 2;\nselect 3;\n");
    }

    #[test]
    fn test_semicolons_in_strings() {
        let input = "SELECT 'hello; world' as test;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(result, "SELECT 'hello; world' as test;\n");

        let input = "SELECT 'test;' as a; SELECT 'another; test' as b;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "SELECT 'test;' as a;\nSELECT 'another; test' as b;\n"
        );

        let input = "SELECT 'it''s; a test' as msg;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(result, "SELECT 'it''s; a test' as msg;\n");

        let input = "BEGIN; UPDATE foo SET bar = 'hello; world'; COMMIT;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "BEGIN;\nUPDATE foo SET bar = 'hello; world';\nCOMMIT;\n"
        );
    }

    #[test]
    fn test_multiline_statements() {
        // Test case that was failing
        let input = "DROP INDEX onek_unique1_constraint;\nALTER TABLE onek RENAME CONSTRAINT onek_unique1_constraint TO onek_unique1_constraint_foo;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "DROP INDEX onek_unique1_constraint;\nALTER TABLE onek RENAME CONSTRAINT onek_unique1_constraint TO onek_unique1_constraint_foo;\n"
        );

        // Test multiline statement without semicolon on first line
        let input = "CREATE TABLE test\n  (id INT PRIMARY KEY);";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(result, "CREATE TABLE test\n  (id INT PRIMARY KEY);\n");
    }

    #[test]
    fn test_multiline_comments() {
        // Test semicolon inside multiline comment
        let input = "/* This is a comment; with semicolon */ SELECT 1;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "/* This is a comment; with semicolon */ SELECT 1;\n"
        );

        // Test multiline comment spanning multiple lines
        let input = "/* not run by default because it requires tr_TR system locale\nSET lc_time TO 'tr_TR'; */\nSELECT 2;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(result, "/* not run by default because it requires tr_TR system locale\nSET lc_time TO 'tr_TR'; */\nSELECT 2;\n");

        // Test multiple statements with multiline comments
        let input = "SELECT 1; /* comment; with; semicolons; */ SELECT 2;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(
            result,
            "SELECT 1;\n/* comment; with; semicolons; */ SELECT 2;\n"
        );

        // Test statement after multiline comment
        let input = "/* comment */ SELECT 3;";
        let result = test_preprocess_sql(input).unwrap();
        assert_eq!(result, "/* comment */ SELECT 3;\n");
    }
}
