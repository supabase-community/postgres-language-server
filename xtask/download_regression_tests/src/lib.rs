use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use regex::Regex;
use std::env;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{BufRead, Cursor, Write};
use std::process::Command;

const OUTPUT_DIR: &str = "crates/pgt_pretty_print/tests/data/multi";

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
        let filename = url.split('/').next_back().unwrap();
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

        let mut processed_content = Vec::new();

        let cursor = Cursor::new(&output.stdout);

        if let Err(e) = preprocess_sql(cursor, &mut processed_content) {
            eprintln!("Error: Failed to process file: {}", e);
            continue;
        }

        let content_str = std::str::from_utf8(&processed_content)?;
        let split_result = pgt_statement_splitter::split(content_str);

        let mut valid_statements = Vec::new();

        for (idx, range) in split_result.ranges.iter().enumerate() {
            let statement = &content_str[usize::from(range.start())..usize::from(range.end())];
            let trimmed = statement.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Try to parse the statement
            match pgt_query::parse(trimmed) {
                Ok(parsed) => {
                    if parsed.root().is_none() {
                        eprintln!(
                            "Warning: Statement {} in {} has no root node, skipping",
                            idx + 1,
                            filename
                        );
                        continue;
                    }

                    // Add valid statement to the list
                    valid_statements.push(trimmed);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse statement {} in {}: {}",
                        idx + 1,
                        filename,
                        e
                    );
                    continue;
                }
            }
        }

        // Write all valid statements to a single file
        if !valid_statements.is_empty() {
            let base_name = filename.strip_suffix(".sql").unwrap_or(filename);
            let output_filename = format!("{}_60.sql", base_name);
            let filepath = target_dir.join(output_filename);

            let mut dest = File::create(&filepath)?;
            for (i, stmt) in valid_statements.iter().enumerate() {
                if i > 0 {
                    write!(dest, "\n\n")?;
                }
                write!(dest, "{}", stmt)?;
            }
            writeln!(dest)?; // Final newline
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

fn preprocess_sql<R: BufRead, W: Write>(source: R, mut dest: W) -> Result<()> {
    let mut skipping_copy_block = false;

    let template_vars_regex = Regex::new(r"^:'([^']+)'|^:([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

    for (idx, line) in source.lines().enumerate() {
        let mut line = line?;

        // Detect the start of the COPY block
        if line.starts_with("COPY ") && line.to_lowercase().contains("from stdin") {
            skipping_copy_block = true;
            continue;
        }

        // Detect the end of the COPY block
        if skipping_copy_block && (line.starts_with("\\.") || line.is_empty()) {
            skipping_copy_block = false;
            continue;
        }

        // Skip lines if inside a COPY block
        if skipping_copy_block {
            continue;
        }

        if line.starts_with("\\") {
            // Skip plpgsql commands (for now)
            continue;
        }

        // replace "\gset" with ";"
        if line.contains("\\gset") {
            line = line.replace("\\gset", ";");
        }

        // Replace template variables
        let mut result = String::new();
        let mut i = 0;
        let bytes = line.as_bytes();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_array = false;

        while i < bytes.len() {
            let c = bytes[i] as char;

            // Handle quote state transitions
            match c {
                '\'' => {
                    result.push(c);
                    i += 1;
                    in_single_quote = !in_single_quote;
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
                ':' if !in_single_quote && !in_double_quote && !in_array => {
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
                        let m = caps.get(1).or_else(|| caps.get(2)).unwrap();
                        let matched_var = &remaining[m.start()..m.end()];

                        println!("#{} Replacing template variable {}", idx, matched_var);

                        result.push('\'');
                        result.push_str(matched_var);
                        result.push('\'');

                        i += full.end();
                        continue;
                    }
                }
                _ => {}
            }

            result.push(c);
            i += 1;
        }

        // Write the cleaned line
        writeln!(dest, "{}", result)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_preprocess_sql(sql: &str) -> Result<String> {
        let input = sql.as_bytes();
        let cursor = Cursor::new(input);
        let mut output = Vec::new();

        preprocess_sql(cursor, &mut output)?;

        Ok(String::from_utf8(output)?)
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
