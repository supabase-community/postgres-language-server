//! The spans the resolver can find in the statements of Postgres' regression suite that parse,
//! as one digest per file and kind of span. Each kind is found at the locations the parser
//! gives for the nodes the resolver finds it for. A change to how any span is found shows up as
//! a changed digest in the snapshot.

use std::fmt::Write;

use pgls_query::{NodeEnum, NodeRef};

use super::{
    alias_span, cast_end, literal_end, operator_end, reference_end, star_end, token_end,
    type_name_end,
};

/// Statements longer than this are skipped. Finding a span can walk to the end of the
/// statement, so the work per statement is quadratic in its length, and the few longer
/// statements of the suite (mostly long string constants) would take most of the time.
const STATEMENT_BYTES_MAX: usize = 10_000;

#[derive(Debug, Clone, Copy)]
enum Kind {
    Name,
    Type,
    Star,
    Operator,
    Cast,
    Literal,
    Alias,
    Expression,
}

impl Kind {
    const ALL: [Kind; 8] = [
        Kind::Name,
        Kind::Type,
        Kind::Star,
        Kind::Operator,
        Kind::Cast,
        Kind::Literal,
        Kind::Alias,
        Kind::Expression,
    ];
}

/// The span of `kind` starting at `start`, the way the resolver finds it.
fn span(sql: &str, kind: Kind, start: usize) -> Option<(usize, usize)> {
    let end = match kind {
        Kind::Name => reference_end(sql, start),
        Kind::Type => type_name_end(sql, start),
        Kind::Star => star_end(sql, start),
        Kind::Operator => operator_end(sql, start),
        Kind::Cast => cast_end(sql, start),
        Kind::Literal => literal_end(sql, start),
        Kind::Alias => return alias_span(sql, start),
        Kind::Expression => reference_end(sql, start).or_else(|| token_end(sql, start)),
    };
    end.map(|end| (start, end))
}

/// The locations in `root` at which the resolver can find each kind of span, in the order of
/// [`Kind::ALL`], sorted and without duplicates.
fn locations(root: &NodeEnum) -> [Vec<usize>; 8] {
    let mut locations: [Vec<i32>; 8] = Default::default();
    for node in root.iter() {
        let (kind, location) = match node {
            NodeRef::TypeName(n) => (Kind::Type, n.location),
            NodeRef::ColumnRef(n)
                if n.fields
                    .last()
                    .is_some_and(|field| matches!(field.node, Some(NodeEnum::AStar(_)))) =>
            {
                (Kind::Star, n.location)
            }
            NodeRef::AExpr(n) => (Kind::Operator, n.location),
            NodeRef::TypeCast(n) => (Kind::Cast, n.location),
            NodeRef::AConst(n) => (Kind::Literal, n.location),
            NodeRef::RangeVar(n) => (Kind::Alias, n.location),
            _ => continue,
        };
        locations[kind as usize].push(location);
    }
    // Names, and the first token of expressions, are found at the locations of many kinds of
    // node, so take every location in the tree. The `Debug` output has them all, as
    // `location: <offset>`.
    let tree = format!("{root:?}");
    let every: Vec<i32> = tree
        .match_indices("location: ")
        .filter(|(index, _)| {
            !tree[..*index].ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        })
        .filter_map(|(index, field)| {
            let digits = &tree[index + field.len()..];
            let len = digits
                .find(|c: char| !(c.is_ascii_digit() || c == '-'))
                .unwrap_or(digits.len());
            digits[..len].parse().ok()
        })
        .collect();
    locations[Kind::Name as usize].clone_from(&every);
    locations[Kind::Expression as usize] = every;
    locations.map(|locations| {
        let mut locations: Vec<usize> = locations
            .into_iter()
            .filter_map(|location| usize::try_from(location).ok())
            .collect();
        locations.sort_unstable();
        locations.dedup();
        locations
    })
}

/// FNV-1a, which unlike the hashers of `std` gives the same digest on every platform and Rust
/// version.
#[derive(Clone, Copy)]
struct Digest(u64);

impl Digest {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn write(&mut self, value: usize) {
        for byte in (value as u64).to_le_bytes() {
            self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }

    fn write_span(&mut self, statement: usize, start: usize, span: Option<(usize, usize)>) {
        self.write(statement);
        self.write(start);
        match span {
            Some((start, end)) => {
                self.write(start);
                self.write(end);
            }
            None => self.write(usize::MAX),
        }
    }
}

#[test]
fn spans_of_the_regression_suite() {
    let version = pgls_postgres_regress::version(18).expect("the fixtures of Postgres 18");
    let mut snapshot = String::new();
    for file in version.files() {
        let mut digests = [Digest::new(); Kind::ALL.len()];
        // The spans found, and the locations tried.
        let mut counts = [(0usize, 0usize); Kind::ALL.len()];
        for (index, statement) in file.statements.iter().enumerate() {
            let sql = file.sql(statement);
            if sql.len() > STATEMENT_BYTES_MAX {
                continue;
            }
            let Some(root) = pgls_query::parse(sql)
                .ok()
                .and_then(|parsed| parsed.into_root())
            else {
                continue;
            };
            for (kind, locations) in Kind::ALL.into_iter().zip(locations(&root)) {
                for start in locations {
                    let span = span(sql, kind, start);
                    digests[kind as usize].write_span(index, start, span);
                    counts[kind as usize].0 += usize::from(span.is_some());
                    counts[kind as usize].1 += 1;
                }
            }
        }
        write!(snapshot, "{}:", file.name).unwrap();
        for ((kind, digest), (found, tried)) in Kind::ALL.iter().zip(digests).zip(counts) {
            write!(
                snapshot,
                " {kind:?} {found}/{tried} {:08x}",
                digest.0 as u32
            )
            .unwrap();
        }
        snapshot.push('\n');
    }
    insta::assert_snapshot!(snapshot);
}
