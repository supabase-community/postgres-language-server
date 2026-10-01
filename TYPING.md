# Static Typing

Plan for static type inference in `pgls_catalog` and the typecheck rules built on it. Paths are relative to `crates/pgls_catalog/src/` unless stated otherwise.

## Goal

Extend the static resolver with types, so the linter reports type errors without a database round trip. Postgres catches these errors when it runs the statement. The static checker matters for the statements EXPLAIN can't check: statements that use objects created earlier in the same file (migrations), and SQL function bodies.

Rules:

| Rule | Postgres error |
| --- | --- |
| `operatorTypeMismatch` | 42883 operator does not exist; 42725 operator is not unique |
| `functionArgumentMismatch` | 42883 function does not exist (the name and argument count exist, the types don't match); 42725 function is not unique |
| `invalidCast` | 42846 cannot cast type X to Y |
| `assignmentTypeMismatch` | 42804 column "c" is of type X but expression is of type Y (INSERT, UPDATE, ON CONFLICT DO UPDATE) |
| `functionReturnTypeMismatch` | 42P13, now also comparing column types |

## Hard Rules

- **No false positives.** Only report a proven incompatibility or ambiguity. Missing metadata, incomplete candidate lists, unmodelled syntax, or an applicable candidate we can't evaluate yield "unknown", and unknown never reports. When in doubt, return `None`.
- **No DDL replay.** Nothing runs against the database in production except the snapshot queries.
- **Follow Postgres, not intuition.** Port the algorithms from `REL_15_STABLE/src/backend`, and name the Postgres function in a comment next to each port. Do not implement transitive cast search or "lowest cost" overload ranking; Postgres does neither.
- **Never compare types by name.** Compare identities (`TypeId`). Names are for lookup and display.
- **Unknown literals are a type.** `'abc'` and `NULL` are `Type::UnknownLiteral` (Postgres' `unknown`), which takes part in overload resolution. `None` means "we don't know" and is different.
- Typmods (`varchar(10)`, `numeric(5,2)`), domain constraints, and invalid literal contents (`'x'::int`) are ignored. They fail at runtime, not at parse time.

## Architecture

```text
typing/
  mod.rs          re-exports
  model.rs        TypeId, Type, TypedColumn, Decision, Selection, CoercionContext, Candidates,
                  TypeInfo metadata, FunctionSignature, OperatorInfo, CastInfo
  normalize.rs    parser TypeName -> TypeId
  coerce.rs       can_coerce, find_coercion_pathway, is_binary_coercible (parse_coerce.c)
  overload.rs     operator and function candidate selection (parse_oper.c, parse_func.c)
  polymorphic.rs  anyelement/anyarray/anycompatible (parse_coerce.c)
  common.rs       select_common_type (parse_coerce.c)
  tests.rs
resolve/expr/     expression typing, one module per expression node
resolve/nodes/    the existing query/scope modules, now carrying column types
snapshot/         casts.rs, operators.rs, extended types.rs/functions.rs + SQL
```

Upgrade the existing pipeline. There is no second resolver: scopes (`resolve/scope.rs`) carry typed columns, and view/CTAS derivation (`catalog/derive.rs`) keeps types.

### Model

```rust
pub enum TypeId {
    /// A type from the database snapshot, by oid.
    Snapshot(i64),
    /// A type created by the current file. Synthetic and stable across renames; a drop and
    /// re-create gets a new id.
    File(u64),
}

pub enum Type {
    /// A named type, including arrays, domains, enums, ranges and composites.
    Named(TypeId),
    /// Postgres' `unknown`: string literals and `NULL` before they are resolved.
    UnknownLiteral,
    /// An anonymous row, e.g. `ROW(1, 'a')`.
    Record(Vec<TypedColumn>),
}

pub struct TypedColumn { pub name: String, pub ty: Option<Type> }

/// Output columns of a query. Outer `None`: unknown shape. Inner `None`: unknown column type.
pub type QueryColumns = Option<Vec<TypedColumn>>;

pub enum Decision<T> { Known(T), Unknown }
pub enum Selection<T> { Match(T), NoMatch, Ambiguous, Unknown }
pub enum CoercionContext { Implicit, Assignment, Explicit }

pub struct Candidates<T> {
    pub items: Vec<T>,
    /// False when the list may be missing candidates (unmodelled DDL, legacy snapshot, ...).
    /// An incomplete list can never prove `NoMatch` or `Ambiguous`.
    pub complete: bool,
}
```

Type metadata (`TypeInfo` gains these, all optional): `kind` (typtype: base, composite, domain, enum, pseudo, range, multirange), `category` (typcategory char), `preferred` (typispreferred), `element` (typelem, for true arrays only: `typsubscript = array_subscript_handler`), `array` (typarray), `base` (typbasetype for domains), `relation` (typrelid). `FunctionInfo` gains `signature: Option<FunctionSignature>`: ordered arguments with name, type, and mode; the number of input defaults; the variadic element type; the return type; `returns_set`. New `OperatorInfo` (oid, schema, name, kind prefix/infix, left, right, result) and `CastInfo` (source, target, context i/a/e, method f/i/b).

`ColumnInfo` gains `ty: Option<Type>`; `type_name` stays for display and compatibility.

### Catalog API

New `CatalogView` methods. Their default implementations return unknown, so test catalogs keep working.

```rust
fn type_by_id(&self, id: &TypeId) -> Lookup<TypeInfo>;
fn cast(&self, source: &TypeId, target: &TypeId) -> Lookup<CastInfo>;
fn function_candidates(&self, schema: Option<&str>, name: &str, search_path: &[String])
    -> Candidates<FunctionInfo>;
fn operator_candidates(&self, schema: Option<&str>, name: &str, kind: OperatorKind,
    search_path: &[String]) -> Candidates<OperatorInfo>;
fn server_version_num(&self) -> Option<i64>;
```

`Lookup::Missing` from `cast` means "no pg_cast row", which is only claimed when the cast list is complete (snapshot has casts, and the file has not run `CREATE CAST`/`DROP CAST` or unmodelled DDL).

**Invalidation affects successful lookups too.** A new cast, operator, or overload can change which existing candidate wins. After unmodelled DDL, `CREATE CAST`, or `CREATE OPERATOR`, candidate lists are incomplete and casts are unknown.

### Engine Entry Points

```rust
// typing/coerce.rs
pub fn can_coerce(c: &dyn CatalogView, from: &Type, to: &TypeId, context: CoercionContext)
    -> Decision<bool>;
// typing/overload.rs
pub fn select_operator(c: &dyn CatalogView, schema: Option<&str>, name: &str,
    left: Option<&Type>, right: &Type, search_path: &[String]) -> Selection<ResolvedOperator>;
pub fn select_function(c: &dyn CatalogView, schema: Option<&str>, name: &str,
    args: &[CallArg], search_path: &[String]) -> Selection<ResolvedFunction>;
// typing/common.rs
pub fn select_common_type(c: &dyn CatalogView, types: &[Type]) -> Selection<Type>;
// resolve/expr/mod.rs
pub(crate) fn infer_expr(r: &mut Resolver<'_>, expr: &NodeEnum) -> Option<Type>;
// resolve (public)
pub fn query_output_types(query: &Node, c: &dyn CatalogView, search_path: &[String]) -> QueryColumns;
```

`ResolvedFunction`/`ResolvedOperator` carry the chosen candidate and the resolved result type (after polymorphic resolution). `CallArg` carries the argument type (`Option<Type>`) and an optional name (`name => value`).

## Postgres Semantics to Port

| Area | Postgres source | Behaviour |
| --- | --- | --- |
| Constants | `parse_node.c: make_const` | int4, then int8, then numeric by size; decimals are numeric; `true`/`false` are bool; strings and NULL are unknown |
| Visibility | `namespace.c: FuncnameGetCandidates, OpernameGetCandidates` | search path masking, named arguments, defaults, variadic expansion |
| Operators | `parse_oper.c: oper, left_oper, binary_oper_exact, oper_select_candidate` | exact match, one unknown side takes the other side's type, domains |
| Functions | `parse_func.c: ParseFuncOrColumn, func_get_detail, func_match_argtypes, func_select_candidate` | exact match, type-name call (`text(x)`, a cast), preferred types and categories, unknown category resolution |
| Coercion | `parse_coerce.c: can_coerce_type, find_coercion_pathway, IsBinaryCoercible` | direct casts, domains, arrays (element pathway), I/O conversion to/from string types, context ordering |
| Polymorphism | `parse_coerce.c: check_generic_type_consistency, enforce_generic_type_consistency` | anyelement/anyarray/anynonarray/anyenum family, anycompatible family; ranges deferred (unknown) |
| Common type | `parse_coerce.c: select_common_type` | same-domain preservation, all unknown gives text, preferred type within a category |
| SQL function returns | `functions.c: check_sql_fn_retval, coerce_fn_result_column` | shape, scalar vs whole-row vs per-column, assignment coercion (PG13+) |

Expression coverage: column refs, whole rows, `$n` and named function parameters, casts, operators, functions (named, default, variadic arguments), CASE, COALESCE, GREATEST/LEAST, NULLIF (via `=` resolution), ARRAY, scalar/array/EXISTS sublinks, IN/ANY/ALL, BETWEEN, IS [NOT] DISTINCT FROM, LIKE/ILIKE/SIMILAR, aggregates and window functions with ordinary signatures, `current_date` and the other SQLValueFunctions, field selection on known composites, array subscripts and slices.

Relation coverage: tables, views, aliases, stars, CTEs, subqueries, RETURNING, function RTEs (with ordinality as int8), VALUES per column, set operations pairwise along the parse tree, JOIN USING (`parse_clause.c: buildMergedJoinVar`).

INSERT targets coerce each value to its column type (`analyze.c: transformInsertStmt`, `parse_target.c: transformAssignedExpr`). Don't infer VALUES on their own before applying the target types.

Unknown for now: recursive CTEs, SEARCH/CYCLE, ordered-set and hypothetical aggregates, range polymorphism, custom subscripting, composite inheritance conversions, row comparisons beyond equality.

## Rules and Fallback

Findings carry the types plus display labels (`format_type` style, e.g. `integer`, `text[]`, `public.mood`). Rules stay thin adapters (see `pgls_analyser/src/lint/typecheck/insert_column_mismatch.rs`). A call never gets both `unknownFunction` and `functionArgumentMismatch`: when the name exists but no overload matches the types, it's `functionArgumentMismatch`; arity-only failures stay `unknownFunction`.

The EXPLAIN fallback stays gated on `database_only`. A static finding already clears `database_only`, so there are no duplicates.

## Testing

- **Unit tests** per module (`typing/tests.rs`, `resolve/tests.rs`, `catalog/tests.rs`) against small handwritten catalogs.
- **Rule specs** (`pgls_analyser/tests/specs/typecheck/<rule>/*.sql`) run against the test database's built-in catalog: the runner loads a `Snapshot` from `postgresql://postgres:postgres@127.0.0.1:5432/postgres` once, keeps only `pg_catalog` and `information_schema` objects, and uses it as the catalog base. No committed fixture. CI runs the tests on Linux, macOS, and Windows, all with Postgres 15.
- **Differential tests** against the real database: for each statement in a corpus, compare the static result with Postgres (Parse/Describe via `PREPARE`, or `CREATE FUNCTION` inside a rolled-back transaction for function bodies). Every statement Postgres accepts must give zero static type errors; every inferred output type must equal Postgres'. Track how often inference is unknown, so a checker that always says unknown fails.

## Work Breakdown

| Worker | Owns | Delivers |
| --- | --- | --- |
| A: model and catalog | `typing/{mod,model,normalize}.rs`, stubs of `typing/{coerce,overload,polymorphic,common}.rs`, `lookup.rs`, `lib.rs`, `snapshot/**`, `catalog/{base,mod}.rs`, `Cargo.toml`, `.sqlx` | Model, snapshot queries and structs, indexed `CatalogView` API, type normalization |
| B: overlay | `catalog/ddl/**`, `catalog/{overlay,names,derive,materialize,tests}.rs` | Types of file-created objects, function replacement by signature, invalidation |
| C: algorithms | `typing/{coerce,overload,polymorphic,common,tests}.rs` | Postgres coercion and overload selection |
| D: resolver | `resolve/**` | Typed scopes, expression typing, typed findings, SQL function return types |
| E: rules | `pgls_analyser` sources and specs, workspace integration, generated files | Four new rules and the extended return rule |
| F: testing | `pgls_analyser/tests/rules_tests.rs`, differential tests | Spec runner on the built-in catalog, differential harness |

Order: A first (contracts); then B, C, D, and F in parallel; then E; then the differential run over everything.
