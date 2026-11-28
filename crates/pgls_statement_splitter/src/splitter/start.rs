use pgls_lexer::SyntaxKind;

use super::{Splitter, is_col_id};

/// A keyword that starts a statement, unless it continues the statement before it, see
/// [`Splitter::statement_start`]. Each has a dedicated splitter function called from
/// [`super::common::statement`].
pub(crate) struct StatementStart {
    kind: SyntaxKind,
    /// Whether it starts a query, which is what follows the CTEs of a `WITH` that continues the
    /// statement, e.g. `PREPARE p AS WITH ... SELECT`.
    pub(crate) query: bool,
    /// The tokens after which it continues the statement, by context: an embedded query, e.g.
    /// `CREATE VIEW v AS SELECT`, or a privilege or event list, e.g. `GRANT INSERT, UPDATE` or
    /// `BEFORE INSERT OR UPDATE`.
    after: &'static [&'static [SyntaxKind]],
    /// The statements in which it never starts one, by their first keyword, see
    /// [`Splitter::unknown`].
    within: &'static [SyntaxKind],
}

/// The query of `CREATE VIEW v AS`, `PREPARE p AS`, `DECLARE c CURSOR FOR` and a rule's `DO
/// [ALSO | INSTEAD]`. `FOR` also precedes the command of a policy, `FOR INSERT`, and a lock,
/// `FOR UPDATE`.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12107
const EMBEDDED_QUERY: &[SyntaxKind] = &[
    SyntaxKind::AS_KW,
    SyntaxKind::FOR_KW,
    SyntaxKind::DO_KW,
    SyntaxKind::ALSO_KW,
    SyntaxKind::INSTEAD_KW,
];

/// A privilege of `GRANT` or `REVOKE`, e.g. `GRANT INSERT, UPDATE`.
const PRIVILEGE: &[SyntaxKind] = &[
    SyntaxKind::GRANT_KW,
    SyntaxKind::REVOKE_KW,
    SyntaxKind::COMMA,
];

/// The event of a rule, `ON INSERT`, or a trigger, `BEFORE INSERT OR UPDATE OF c` or `INSTEAD OF
/// INSERT`; a `MERGE` action, `WHEN MATCHED THEN UPDATE`; a lock, `FOR NO KEY UPDATE` or `LOCK t
/// IN SHARE UPDATE EXCLUSIVE MODE`.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12469
const DML_EVENT: &[SyntaxKind] = &[
    SyntaxKind::ON_KW,
    SyntaxKind::BEFORE_KW,
    SyntaxKind::AFTER_KW,
    SyntaxKind::OR_KW,
    SyntaxKind::OF_KW,
    SyntaxKind::THEN_KW,
    SyntaxKind::KEY_KW,
    SyntaxKind::SHARE_KW,
];

/// The objects of `ALTER`, e.g. `ALTER TABLE`, `ALTER PROCEDURAL LANGUAGE` or `ALTER LARGE OBJECT`.
/// They're followed by the object's name, so a statement keyword after one is a name, e.g. in
/// `CREATE DOMAIN update text`, see [`Splitter::at_operand`].
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L9386
pub(super) const ALTER_OBJECTS: &[SyntaxKind] = &[
    SyntaxKind::AGGREGATE_KW,
    SyntaxKind::COLLATION_KW,
    SyntaxKind::CONVERSION_KW,
    SyntaxKind::DATABASE_KW,
    SyntaxKind::DEFAULT_KW,
    SyntaxKind::DOMAIN_KW,
    SyntaxKind::EVENT_KW,
    SyntaxKind::EXTENSION_KW,
    SyntaxKind::FOREIGN_KW,
    SyntaxKind::FUNCTION_KW,
    SyntaxKind::GROUP_KW,
    SyntaxKind::INDEX_KW,
    SyntaxKind::LANGUAGE_KW,
    SyntaxKind::LARGE_KW,
    SyntaxKind::MATERIALIZED_KW,
    SyntaxKind::OPERATOR_KW,
    SyntaxKind::POLICY_KW,
    SyntaxKind::PROCEDURAL_KW,
    SyntaxKind::PROCEDURE_KW,
    SyntaxKind::PUBLICATION_KW,
    SyntaxKind::ROLE_KW,
    SyntaxKind::ROUTINE_KW,
    SyntaxKind::RULE_KW,
    SyntaxKind::SCHEMA_KW,
    SyntaxKind::SEQUENCE_KW,
    SyntaxKind::SERVER_KW,
    SyntaxKind::STATISTICS_KW,
    SyntaxKind::SUBSCRIPTION_KW,
    SyntaxKind::SYSTEM_KW,
    SyntaxKind::TABLE_KW,
    SyntaxKind::TABLESPACE_KW,
    SyntaxKind::TEXT_KW,
    SyntaxKind::TRIGGER_KW,
    SyntaxKind::TYPE_KW,
    SyntaxKind::USER_KW,
    SyntaxKind::VIEW_KW,
];

static STATEMENT_STARTS: &[StatementStart] = &[
    StatementStart {
        kind: SyntaxKind::WITH_KW,
        query: false,
        after: &[EMBEDDED_QUERY],
        // The query of `INSERT INTO`, and a clause of `ALTER`, e.g. `ALTER ROLE r WITH ...`.
        within: &[SyntaxKind::INSERT_KW, SyntaxKind::ALTER_KW],
    },
    StatementStart {
        kind: SyntaxKind::SELECT_KW,
        query: true,
        // A set operation, `UNION [ALL | DISTINCT]`, `INTERSECT` or `EXCEPT`, a `BEGIN ATOMIC`
        // body, or the event of a rule.
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12935
        after: &[
            EMBEDDED_QUERY,
            PRIVILEGE,
            &[
                SyntaxKind::UNION_KW,
                SyntaxKind::ALL_KW,
                SyntaxKind::DISTINCT_KW,
                SyntaxKind::INTERSECT_KW,
                SyntaxKind::EXCEPT_KW,
                SyntaxKind::ATOMIC_KW,
                SyntaxKind::ON_KW,
            ],
        ],
        // The query of `INSERT INTO`.
        within: &[SyntaxKind::INSERT_KW],
    },
    StatementStart {
        kind: SyntaxKind::INSERT_KW,
        query: true,
        after: &[EMBEDDED_QUERY, PRIVILEGE, DML_EVENT],
        within: &[],
    },
    StatementStart {
        kind: SyntaxKind::UPDATE_KW,
        query: true,
        after: &[EMBEDDED_QUERY, PRIVILEGE, DML_EVENT],
        within: &[],
    },
    StatementStart {
        kind: SyntaxKind::DELETE_KW,
        query: true,
        after: &[EMBEDDED_QUERY, PRIVILEGE, DML_EVENT],
        within: &[],
    },
    StatementStart {
        kind: SyntaxKind::CREATE_KW,
        query: false,
        after: &[PRIVILEGE],
        // The elements of `CREATE SCHEMA s CREATE TABLE t (...) CREATE VIEW ...`, which psql
        // sends as one statement, up to the `;`.
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L1517
        within: &[SyntaxKind::SCHEMA_KW],
    },
    StatementStart {
        kind: SyntaxKind::ALTER_KW,
        query: false,
        // A privilege, e.g. `REVOKE [GRANT OPTION FOR] ALTER SYSTEM ON PARAMETER ...`.
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L7754
        after: &[&[
            SyntaxKind::GRANT_KW,
            SyntaxKind::REVOKE_KW,
            SyntaxKind::FOR_KW,
        ]],
        // A clause of `ALTER`, e.g. `ALTER TABLE t ALTER COLUMN c ...`.
        within: &[SyntaxKind::ALTER_KW],
    },
];

impl StatementStart {
    /// Whether the keyword continues the statement after the token `prev`, or as the query after
    /// CTEs if `cte_query`.
    pub(crate) fn continues_after(&self, prev: Option<SyntaxKind>, cte_query: bool) -> bool {
        prev.is_some_and(|prev| self.after.iter().any(|tokens| tokens.contains(&prev)))
            || (self.query && cte_query)
    }
}

impl Splitter<'_> {
    /// The statement keyword at the current token, if it can start a statement in the statements
    /// `within`, see [`Splitter::unknown`]: it's in [`STATEMENT_STARTS`] and no token after it
    /// makes it something else.
    pub(crate) fn statement_start(&self, within: &[SyntaxKind]) -> Option<&'static StatementStart> {
        let kind = self.current();
        let start = STATEMENT_STARTS.iter().find(|start| start.kind == kind)?;
        let vetoed = start
            .within
            .iter()
            .any(|statement| within.contains(statement))
            || match (kind, self.look_ahead()) {
                // A keyword before a `.` qualifies a name, e.g. the schema in `alter.c`.
                (_, SyntaxKind::DOT) => true,
                // The unreserved statement keywords are names too, e.g. the type in `CREATE TYPE
                // insert AS ...`, a table alias, hstore's delete() function, or in `ON COMMIT
                // DELETE ROWS`. They start a statement only before the token that has to follow
                // them there: `INSERT INTO`, `DELETE FROM`, `UPDATE [ONLY] name` and `ALTER
                // <object>`. A `SET` after `UPDATE` is either the name or the SET clause after
                // an alias, see [`Splitter::at_set_clause`].
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12239
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12427
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12497
                (SyntaxKind::INSERT_KW, next) => next != SyntaxKind::INTO_KW,
                (SyntaxKind::DELETE_KW, next) => next != SyntaxKind::FROM_KW,
                (SyntaxKind::UPDATE_KW, SyntaxKind::SET_KW) => self.at_set_clause(),
                (SyntaxKind::UPDATE_KW, next) => next != SyntaxKind::ONLY_KW && !is_col_id(next),
                (SyntaxKind::ALTER_KW, next) => !ALTER_OBJECTS.contains(&next),
                // Part of a clause, e.g. `WITH ORDINALITY`, `WITH CHECK` or `WITH TIME ZONE`.
                (SyntaxKind::WITH_KW, _) => !self.at_cte(),
                _ => false,
            };
        (!vetoed).then_some(start)
    }

    /// Whether the `SET` after the `UPDATE` at the current token starts the SET clause of an
    /// `UPDATE`, which makes `update` the alias of its table, e.g. `UPDATE t update SET a = 1`,
    /// rather than the table `set`, e.g. `UPDATE set SET a = 1`. A SET clause goes on with `(` or
    /// a column and `=`, `[` or `.`, neither of which can follow the table `set`. Looks at most
    /// three tokens ahead.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12520
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L13976
    fn at_set_clause(&self) -> bool {
        match self.nth(2) {
            SyntaxKind::L_PAREN => true,
            column if is_col_id(column) => matches!(
                self.nth(3),
                SyntaxKind::EQ | SyntaxKind::L_BRACK | SyntaxKind::DOT
            ),
            _ => false,
        }
    }

    /// Whether the `WITH` at the current token starts common table expressions, rather than
    /// being part of a clause like `COPY ... WITH (...)`, `DECLARE ... WITH HOLD` or `WITH NO
    /// DATA`. A CTE is `WITH RECURSIVE`, `WITH name (` or `WITH name AS [NOT] MATERIALIZED (`. The
    /// end of the text counts as a match, so a half-written CTE still starts a statement. Looks
    /// at most three tokens ahead.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L13040
    fn at_cte(&self) -> bool {
        match self.nth(1) {
            SyntaxKind::RECURSIVE_KW | SyntaxKind::EOF => true,
            name if is_col_id(name) => match self.nth(2) {
                SyntaxKind::L_PAREN | SyntaxKind::EOF => true,
                SyntaxKind::AS_KW => matches!(
                    self.nth(3),
                    SyntaxKind::L_PAREN
                        | SyntaxKind::NOT_KW
                        | SyntaxKind::MATERIALIZED_KW
                        | SyntaxKind::EOF
                ),
                _ => false,
            },
            _ => false,
        }
    }
}
