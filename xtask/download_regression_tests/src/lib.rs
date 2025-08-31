use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use regex::Regex;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{BufRead, Cursor, Write};
use std::process::Command;

const OUTPUT_DIR: &str = "crates/pgt_pretty_print/tests/data/regression_suite";

pub fn download_regression_tests() -> Result<()> {
    let target_dir = Utf8PathBuf::from(OUTPUT_DIR);

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
        let filepath = target_dir.join(filename);

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

        let mut dest = File::create(&filepath)?;
        dest.write_all(&processed_content)?
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
    let dollar_quote_regex = Regex::new(r"\$([^$]*)\$").unwrap();

    let mut current_statement = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_quote_tag = String::new();

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

        // remove everything after -- in the line (inline comments)
        if let Some(pos) = result.find("--") {
            result.truncate(pos);
        }

        // Check for dollar quotes in the current line
        let mut chars = result.chars().peekable();
        let mut i = 0;
        while i < result.len() {
            if let Some(caps) = dollar_quote_regex.find_at(&result, i) {
                let tag = caps.as_str();
                if in_dollar_quote {
                    // Check if this closes the current dollar quote
                    if tag == dollar_quote_tag {
                        in_dollar_quote = false;
                        dollar_quote_tag.clear();
                    }
                } else {
                    // Opening a new dollar quote
                    in_dollar_quote = true;
                    dollar_quote_tag = tag.to_string();
                }
                i += tag.len();
            } else {
                i += 1;
            }
        }

        current_statement.push_str(&result);
        current_statement.push('\n');

        // Only check for semicolon if we're not inside a dollar quote
        if !in_dollar_quote && current_statement.trim().ends_with(';') {
            let result = pgt_query::parse(&current_statement);
            match result {
                Ok(r) => {
                    if r.root().is_none() {
                        bail!(
                            "Parsed SQL statement at line {} has no root node:\nStatement:\n{}",
                            idx + 1,
                            current_statement
                        );
                    }
                }
                Err(e) => bail!(
                    "Failed to parse SQL statement at line {}: {}\nStatement:\n{}",
                    idx + 1,
                    e,
                    current_statement
                ),
            }
            println!("Writing statement {}", current_statement.trim());
            writeln!(dest, "{}", current_statement.trim())?;
            current_statement.clear();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_preprocess_sql(sql: &str) -> Result<String> {
        let input = sql.as_bytes();
        let mut output = Vec::new();
        let cursor = Cursor::new(input);
        preprocess_sql(cursor, &mut output)?;
        String::from_utf8(output).map_err(Into::into)
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
            ("d := $1::di;", "d := $1::di;"),
            (
                "SELECT JSON_OBJECT('foo': NULL::int FORMAT JSON);",
                "SELECT JSON_OBJECT('foo': NULL::int FORMAT JSON);",
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
}
