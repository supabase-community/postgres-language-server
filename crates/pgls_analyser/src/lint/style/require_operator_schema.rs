use crate::fix::StatementTokens;
use crate::utils::statement_nodes;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeRef;
use pgls_query::protobuf::{AExprKind, Node, SubLinkType, Token};
use pgls_query_ext::utils::string_value;
use pgls_text_size::TextRange;

/// The most tokens of a construct that uses an operator, like `IS NOT DISTINCT FROM`.
const CONSTRUCT_TOKENS_MAX: usize = 4;

declare_lint_rule! {
    /// Operators should be schema-qualified.
    ///
    /// Without a schema, Postgres looks the operator up in every schema on the `search_path`
    /// and uses the best match. An operator with the same name in a schema earlier on the
    /// search path, or one in any of them whose argument types match better, is used instead of
    /// the intended one. Anyone who can create operators in one of these schemas can run code
    /// with the privileges of the caller. A schema-qualified operator is written as
    /// `OPERATOR(schema.op)`, e.g. `a OPERATOR(pg_catalog.=) b`.
    ///
    /// The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
    /// extension scripts, as defence in depth next to setting the `search_path` explicitly.
    ///
    /// The rule checks binary and unary operators, `op ANY (...)` and `op ALL (...)` with arrays
    /// and subqueries, and `ORDER BY ... USING op`. It also reports the SQL constructs that
    /// Postgres turns into operators that it looks up through the search path: `IN` and
    /// `NOT IN` (`=` and `<>`), `LIKE` and `ILIKE` (`~~` and `~~*`), `SIMILAR TO` (`~`),
    /// `IS [NOT] DISTINCT FROM` and `NULLIF` (`=`), and `BETWEEN` (`>=` and `<=`). They have no
    /// schema-qualified form, so the comparison has to be written with `OPERATOR(...)` instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select * from pg_catalog.pg_stat_io where backend_type = 'client backend';
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// select * from pg_catalog.pg_class where relkind in ('r', 'p');
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select * from pg_catalog.pg_stat_io where backend_type operator(pg_catalog.=) 'client backend';
    /// select * from pg_catalog.pg_class where relkind operator(pg_catalog.=) any (array['r', 'p']);
    /// ```
    ///
    pub RequireOperatorSchema {
        version: "next",
        name: "requireOperatorSchema",
        severity: Severity::Warning,
        recommended: false,
    }
}

impl LinterRule for RequireOperatorSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        let mut lookups: Vec<Lookup> = statement_nodes(ctx.stmt())
            .into_iter()
            .filter_map(lookup)
            .collect();
        lookups.sort_by_key(|lookup| lookup.location);
        lookups
            .into_iter()
            .map(|lookup| {
                let span = tokens.and_then(|tokens| lookup.span(tokens));
                let operator = lookup.operator;
                let diagnostic = match lookup.construct {
                    None => LinterDiagnostic::new(
                        rule_category!(),
                        span,
                        markup! {
                            "Operator "<Emphasis>{operator}</Emphasis>" is not schema-qualified."
                        },
                    ),
                    Some(construct) => {
                        let (noun, operators) = match lookup.second_operator {
                            None => ("operator", operator.to_owned()),
                            Some(second) => ("operators", format!("{operator} and {second}")),
                        };
                        LinterDiagnostic::new(
                            rule_category!(),
                            span,
                            markup! {
                                <Emphasis>{construct.name}</Emphasis>" uses the "{noun}" "<Emphasis>{operators}</Emphasis>" without a schema."
                            },
                        )
                    }
                };
                let diagnostic = diagnostic.detail(
                    None,
                    "Without a schema, the operator is resolved through the search_path: an operator with the same name earlier on the search_path, or one whose argument types match better, is used instead.",
                );
                match lookup.construct {
                    None => diagnostic.note(markup! {
                        "Specify the schema, e.g. "<Emphasis>"OPERATOR(pg_catalog."{operator}")"</Emphasis>"."
                    }),
                    Some(construct) => diagnostic.note(markup! {
                        <Emphasis>{construct.name}</Emphasis>" has no schema-qualified form. Write the comparison with "<Emphasis>"OPERATOR(pg_catalog."{operator}")"</Emphasis>" instead."
                    }),
                }
            })
            .collect()
    }
}

/// An operator that Postgres looks up through the search path.
struct Lookup<'a> {
    operator: &'a str,
    /// The other operator of `BETWEEN`, which uses `>=` and `<=`.
    second_operator: Option<&'a str>,
    /// The SQL construct that uses the operators, if they are not written out.
    construct: Option<Construct>,
    location: i32,
}

impl Lookup<'_> {
    /// The operator, or the keywords of the construct, e.g. `IS NOT DISTINCT FROM`.
    fn span(&self, tokens: StatementTokens) -> Option<TextRange> {
        let start = tokens.at(self.location)?;
        let Some(construct) = self.construct else {
            return Some(tokens.range(start));
        };
        let mut end = (start..start + CONSTRUCT_TOKENS_MAX)
            .find(|index| tokens.kind(*index) == Some(construct.last))
            .unwrap_or(start);
        if construct.last == Token::Between
            && matches!(
                tokens.kind(end + 1),
                Some(Token::Symmetric | Token::Asymmetric)
            )
        {
            end += 1;
        }
        Some(tokens.range(start).cover(tokens.range(end)))
    }
}

/// A SQL construct that Postgres turns into operators, like `IN`.
#[derive(Clone, Copy)]
struct Construct {
    name: &'static str,
    /// The last keyword of the construct, e.g. `FROM` of `IS DISTINCT FROM`.
    last: Token,
}

impl Construct {
    const fn new(name: &'static str, last: Token) -> Self {
        Self { name, last }
    }
}

/// The operator lookup of a node, if Postgres looks up an operator through the search path.
fn lookup(node: NodeRef) -> Option<Lookup> {
    let (name, construct, location) = match node {
        NodeRef::AExpr(expr) => {
            // The negated constructs, like `NOT IN`, use the negated operator, like `<>`.
            let negated = matches!(unqualified(&expr.name), Some("<>" | "!~~" | "!~~*" | "!~"));
            let construct = match expr.kind() {
                AExprKind::AexprOp | AExprKind::AexprOpAny | AExprKind::AexprOpAll => None,
                AExprKind::AexprIn if negated => Some(Construct::new("NOT IN", Token::InP)),
                AExprKind::AexprIn => Some(Construct::new("IN", Token::InP)),
                AExprKind::AexprLike if negated => Some(Construct::new("NOT LIKE", Token::Like)),
                AExprKind::AexprLike => Some(Construct::new("LIKE", Token::Like)),
                AExprKind::AexprIlike if negated => Some(Construct::new("NOT ILIKE", Token::Ilike)),
                AExprKind::AexprIlike => Some(Construct::new("ILIKE", Token::Ilike)),
                AExprKind::AexprSimilar if negated => {
                    Some(Construct::new("NOT SIMILAR TO", Token::To))
                }
                AExprKind::AexprSimilar => Some(Construct::new("SIMILAR TO", Token::To)),
                AExprKind::AexprDistinct => Some(Construct::new("IS DISTINCT FROM", Token::From)),
                AExprKind::AexprNotDistinct => {
                    Some(Construct::new("IS NOT DISTINCT FROM", Token::From))
                }
                AExprKind::AexprNullif => Some(Construct::new("NULLIF", Token::Nullif)),
                AExprKind::AexprBetween
                | AExprKind::AexprBetweenSym
                | AExprKind::AexprNotBetween
                | AExprKind::AexprNotBetweenSym => {
                    // The name is the construct, e.g. `NOT BETWEEN`, which compares with `<`
                    // and `>`.
                    let name = match expr.kind() {
                        AExprKind::AexprNotBetween | AExprKind::AexprNotBetweenSym => "NOT BETWEEN",
                        _ => "BETWEEN",
                    };
                    let (operator, second) = match expr.kind() {
                        AExprKind::AexprNotBetween | AExprKind::AexprNotBetweenSym => ("<", ">"),
                        _ => (">=", "<="),
                    };
                    return Some(Lookup {
                        operator,
                        second_operator: Some(second),
                        construct: Some(Construct::new(name, Token::Between)),
                        location: expr.location,
                    });
                }
                AExprKind::Undefined => return None,
            };
            (unqualified(&expr.name)?, construct, expr.location)
        }
        NodeRef::SubLink(sub_link) => match sub_link.sub_link_type() {
            // `IN (subquery)` has no operator name and compares with `=`.
            SubLinkType::AnySublink if sub_link.oper_name.is_empty() => (
                "=",
                Some(Construct::new("IN", Token::InP)),
                sub_link.location,
            ),
            SubLinkType::AnySublink | SubLinkType::AllSublink => {
                (unqualified(&sub_link.oper_name)?, None, sub_link.location)
            }
            _ => return None,
        },
        NodeRef::SortBy(sort_by) => (unqualified(&sort_by.use_op)?, None, sort_by.location),
        _ => return None,
    };
    Some(Lookup {
        operator: name,
        second_operator: None,
        construct,
        location,
    })
}

/// The name of an operator that is not schema-qualified.
fn unqualified(name: &[Node]) -> Option<&str> {
    match name {
        [name] => string_value(name),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireOperatorSchema", sql)
    }

    #[test]
    fn points_at_the_operator() {
        assert_eq!(
            spans("select a = b, -a, a operator(=) b, a operator(pg_catalog.=) b, a != b from t"),
            ["=", "-", "operator", "!="]
        );
        assert_eq!(
            spans("select a = any (array[1]), a < all (select b from u) from t"),
            ["=", "<"]
        );
        assert_eq!(spans("select a from t order by a using <"), ["<"]);
    }

    #[test]
    fn points_at_the_construct() {
        assert_eq!(
            spans(
                "select a in (1), a not in (1), a in (select b from u), a like 'x', \
                 a not ilike 'x', a similar to 'x', a is not distinct from b, nullif(a, b), \
                 a not between symmetric 1 and 2 from t"
            ),
            [
                "in",
                "not in",
                "in",
                "like",
                "not ilike",
                "similar to",
                "is not distinct from",
                "nullif",
                "not between symmetric"
            ]
        );
    }
}
