use pgls_query::protobuf::Token;

/// Checks the whitespace the formatter wrote between tokens: no space before a comma or `)`, none
/// after `(`, no double space between two tokens on a line, no trailing whitespace and no blank
/// line. Whitespace inside string literals, quoted identifiers and comments is left alone, and so
/// is the space that separates a comment from a parenthesis or comma.
pub fn whitespace_problems(output: &str) -> Vec<String> {
    let Ok(scan) = pgls_query::scan(output) else {
        // The callers check that the output parses.
        return Vec::new();
    };
    let tokens: Vec<_> = scan
        .tokens
        .iter()
        .map(|token| {
            (
                Token::try_from(token.token).ok(),
                token.start as usize,
                token.end as usize,
            )
        })
        .collect();
    let line_at = |offset: usize| {
        let number = output[..offset].matches('\n').count() + 1;
        let start = output[..offset].rfind('\n').map_or(0, |i| i + 1);
        let end = output[offset..]
            .find('\n')
            .map_or(output.len(), |i| offset + i);
        format!("line {number} `{}`", &output[start..end])
    };
    let is_comment =
        |token: Option<Token>| matches!(token, Some(Token::SqlComment | Token::CComment));
    let mut problems = Vec::new();

    for pair in tokens.windows(2) {
        let [(left, _, left_end), (right, right_start, _)] = pair else {
            unreachable!()
        };
        let gap = &output[*left_end..*right_start];
        if gap.is_empty() || gap.contains('\n') {
            continue;
        }
        let problem = if gap.len() > 1 {
            "double space"
        } else if is_comment(*left) || is_comment(*right) {
            continue;
        } else if matches!(left, Some(Token::Ascii40)) {
            "space after `(`"
        } else if matches!(right, Some(Token::Ascii41)) {
            "space before `)`"
        } else if matches!(right, Some(Token::Ascii44)) {
            "space before `,`"
        } else {
            continue;
        };
        problems.push(format!("{problem} in {}", line_at(*left_end)));
    }

    // Whitespace at the end of a line, or a whole line of it, unless it is inside a token such as
    // a multi-line string.
    let inside_token = |offset: usize| {
        tokens
            .iter()
            .any(|(_, start, end)| *start <= offset && offset < *end)
    };
    let lines: Vec<_> = output.split('\n').collect();
    let mut line_start = 0;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_end_matches([' ', '\t']);
        if trimmed.len() < line.len() && !inside_token(line_start + trimmed.len()) {
            problems.push(format!(
                "trailing whitespace in line {} `{line}`",
                index + 1
            ));
        } else if line.is_empty()
            && index > 0
            && index + 1 < lines.len()
            && !inside_token(line_start - 1)
        {
            problems.push(format!("blank line at line {}", index + 1));
        }
        line_start += line.len() + 1;
    }

    problems
}

#[allow(dead_code)] // Not every test crate asserts directly.
pub fn assert_clean_whitespace(output: &str) {
    let problems = whitespace_problems(output);
    assert!(
        problems.is_empty(),
        "stray whitespace in the output:\n{}\n\n{output}",
        problems.join("\n")
    );
}
