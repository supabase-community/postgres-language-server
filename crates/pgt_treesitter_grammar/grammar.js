/**
 * @file A grammar specifically designed for use with the Postgres Language Server by Supabase-Community. It is tailored to provide autocompletions and other LSP features.
 * Based on tree-sitter-sql by Derek Stride (https://github.com/DerekStride/tree-sitter-sql)
 * Original work licensed under MIT License
 * @author juleswritescode
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "pgls",

  extras: ($) => [/\s\n/, /\s/, $.comment, $.marginalia],

  externals: ($) => [
    $._dollar_quoted_string_start_tag,
    $._dollar_quoted_string_end_tag,
    $._dollar_quoted_string,
  ],

  conflicts: ($) => [
    [$.all_fields, $.field_qualifier],
    [$.object_reference, $._qualified_field],
    [$.object_reference],
    [$.between_expression, $.binary_expression],
    [$.time],
    [$.timestamp],
    [$.grantable_on_function, $.grantable_on_table],
    [$.any_identifier, $.column_identifier],
  ],

  precedences: ($) => [
    [
      "binary_is",
      "unary_not",
      "binary_exp",
      "binary_times",
      "binary_plus",
      "unary_other",
      "binary_other",
      "binary_in",
      "binary_compare",
      "binary_relation",
      "pattern_matching",
      "between",
      "clause_connective",
      "clause_disjunctive",
    ],
  ],

  word: ($) => $._identifier,

  rules: {
    program: ($) =>
      seq(
        // any number of transactions, statements, or blocks with a terminating ;
        repeat(seq(choice($.transaction, $.statement, $.block), ";")),
        // optionally, a single statement without a terminating ;
        optional($.statement)
      ),

    keyword_select: (_) => make_keyword("select"),
    keyword_delete: (_) => make_keyword("delete"),
    keyword_insert: (_) => make_keyword("insert"),
    keyword_replace: (_) => make_keyword("replace"),
    keyword_update: (_) => make_keyword("update"),
    keyword_truncate: (_) => make_keyword("truncate"),
    keyword_merge: (_) => make_keyword("merge"),
    keyword_show: (_) => make_keyword("show"),
    keyword_unload: (_) => make_keyword("unload"),
    keyword_into: (_) => make_keyword("into"),
    keyword_overwrite: (_) => make_keyword("overwrite"),
    keyword_values: (_) => make_keyword("values"),
    keyword_value: (_) => make_keyword("value"),
    keyword_matched: (_) => make_keyword("matched"),
    keyword_set: (_) => make_keyword("set"),
    keyword_from: (_) => make_keyword("from"),
    keyword_left: (_) => make_keyword("left"),
    keyword_right: (_) => make_keyword("right"),
    keyword_inner: (_) => make_keyword("inner"),
    keyword_full: (_) => make_keyword("full"),
    keyword_outer: (_) => make_keyword("outer"),
    keyword_cross: (_) => make_keyword("cross"),
    keyword_join: (_) => make_keyword("join"),
    keyword_lateral: (_) => make_keyword("lateral"),
    keyword_natural: (_) => make_keyword("natural"),
    keyword_on: (_) => make_keyword("on"),
    keyword_off: (_) => make_keyword("off"),
    keyword_where: (_) => make_keyword("where"),
    keyword_order: (_) => make_keyword("order"),
    keyword_group: (_) => make_keyword("group"),
    keyword_partition: (_) => make_keyword("partition"),
    keyword_by: (_) => make_keyword("by"),
    keyword_having: (_) => make_keyword("having"),
    keyword_desc: (_) => make_keyword("desc"),
    keyword_asc: (_) => make_keyword("asc"),
    keyword_limit: (_) => make_keyword("limit"),
    keyword_offset: (_) => make_keyword("offset"),
    keyword_primary: (_) => make_keyword("primary"),
    keyword_create: (_) => make_keyword("create"),
    keyword_alter: (_) => make_keyword("alter"),
    keyword_change: (_) => make_keyword("change"),
    keyword_analyze: (_) => make_keyword("analyze"),
    keyword_explain: (_) => make_keyword("explain"),
    keyword_verbose: (_) => make_keyword("verbose"),
    keyword_modify: (_) => make_keyword("modify"),
    keyword_drop: (_) => make_keyword("drop"),
    keyword_add: (_) => make_keyword("add"),
    keyword_table: (_) => make_keyword("table"),
    keyword_tables: (_) => make_keyword("tables"),
    keyword_view: (_) => make_keyword("view"),
    keyword_column: (_) => make_keyword("column"),
    keyword_columns: (_) => make_keyword("columns"),
    keyword_materialized: (_) => make_keyword("materialized"),
    keyword_tablespace: (_) => make_keyword("tablespace"),
    keyword_sequence: (_) => make_keyword("sequence"),
    keyword_increment: (_) => make_keyword("increment"),
    keyword_minvalue: (_) => make_keyword("minvalue"),
    keyword_maxvalue: (_) => make_keyword("maxvalue"),
    keyword_none: (_) => make_keyword("none"),
    keyword_owned: (_) => make_keyword("owned"),
    keyword_start: (_) => make_keyword("start"),
    keyword_restart: (_) => make_keyword("restart"),
    keyword_key: (_) => make_keyword("key"),
    keyword_duplicate: (_) => make_keyword("duplicate"),
    keyword_as: (_) => make_keyword("as"),
    keyword_distinct: (_) => make_keyword("distinct"),
    keyword_constraint: (_) => make_keyword("constraint"),
    keyword_filter: (_) => make_keyword("filter"),
    keyword_cast: (_) => make_keyword("cast"),
    keyword_separator: (_) => make_keyword("separator"),
    keyword_case: (_) => make_keyword("case"),
    keyword_when: (_) => make_keyword("when"),
    keyword_then: (_) => make_keyword("then"),
    keyword_else: (_) => make_keyword("else"),
    keyword_end: (_) => make_keyword("end"),
    keyword_in: (_) => make_keyword("in"),
    keyword_and: (_) => make_keyword("and"),
    keyword_or: (_) => make_keyword("or"),
    keyword_is: (_) => make_keyword("is"),
    keyword_not: (_) => make_keyword("not"),
    keyword_force: (_) => make_keyword("force"),
    keyword_ignore: (_) => make_keyword("ignore"),
    keyword_using: (_) => make_keyword("using"),
    keyword_use: (_) => make_keyword("use"),
    keyword_index: (_) => make_keyword("index"),
    keyword_for: (_) => make_keyword("for"),
    keyword_if: (_) => make_keyword("if"),
    keyword_exists: (_) => make_keyword("exists"),
    keyword_auto_increment: (_) => make_keyword("auto_increment"),
    keyword_generated: (_) => make_keyword("generated"),
    keyword_always: (_) => make_keyword("always"),
    keyword_collate: (_) => make_keyword("collate"),
    keyword_character: (_) => make_keyword("character"),
    keyword_engine: (_) => make_keyword("engine"),
    keyword_default: (_) => make_keyword("default"),
    keyword_cascade: (_) => make_keyword("cascade"),
    keyword_restrict: (_) => make_keyword("restrict"),
    keyword_with: (_) => make_keyword("with"),
    keyword_without: (_) => make_keyword("without"),
    keyword_no: (_) => make_keyword("no"),
    keyword_data: (_) => make_keyword("data"),
    keyword_type: (_) => make_keyword("type"),
    keyword_rename: (_) => make_keyword("rename"),
    keyword_to: (_) => make_keyword("to"),
    keyword_database: (_) => make_keyword("database"),
    keyword_schema: (_) => make_keyword("schema"),
    keyword_owner: (_) => make_keyword("owner"),
    keyword_user: (_) => make_keyword("user"),
    keyword_admin: (_) => make_keyword("admin"),
    keyword_password: (_) => make_keyword("password"),
    keyword_encrypted: (_) => make_keyword("encrypted"),
    keyword_valid: (_) => make_keyword("valid"),
    keyword_until: (_) => make_keyword("until"),
    keyword_connection: (_) => make_keyword("connection"),
    keyword_role: (_) => make_keyword("role"),
    keyword_reset: (_) => make_keyword("reset"),
    keyword_temp: (_) => make_keyword("temp"),
    keyword_temporary: (_) => make_keyword("temporary"),
    keyword_unlogged: (_) => make_keyword("unlogged"),
    keyword_logged: (_) => make_keyword("logged"),
    keyword_cycle: (_) => make_keyword("cycle"),
    keyword_union: (_) => make_keyword("union"),
    keyword_all: (_) => make_keyword("all"),
    keyword_any: (_) => make_keyword("any"),
    keyword_some: (_) => make_keyword("some"),
    keyword_except: (_) => make_keyword("except"),
    keyword_intersect: (_) => make_keyword("intersect"),
    keyword_returning: (_) => make_keyword("returning"),
    keyword_begin: (_) => make_keyword("begin"),
    keyword_commit: (_) => make_keyword("commit"),
    keyword_rollback: (_) => make_keyword("rollback"),
    keyword_transaction: (_) => make_keyword("transaction"),
    keyword_over: (_) => make_keyword("over"),
    keyword_nulls: (_) => make_keyword("nulls"),
    keyword_first: (_) => make_keyword("first"),
    keyword_after: (_) => make_keyword("after"),
    keyword_before: (_) => make_keyword("before"),
    keyword_last: (_) => make_keyword("last"),
    keyword_window: (_) => make_keyword("window"),
    keyword_range: (_) => make_keyword("range"),
    keyword_rows: (_) => make_keyword("rows"),
    keyword_groups: (_) => make_keyword("groups"),
    keyword_between: (_) => make_keyword("between"),
    keyword_unbounded: (_) => make_keyword("unbounded"),
    keyword_preceding: (_) => make_keyword("preceding"),
    keyword_following: (_) => make_keyword("following"),
    keyword_exclude: (_) => make_keyword("exclude"),
    keyword_current: (_) => make_keyword("current"),
    keyword_row: (_) => make_keyword("row"),
    keyword_ties: (_) => make_keyword("ties"),
    keyword_others: (_) => make_keyword("others"),
    keyword_only: (_) => make_keyword("only"),
    keyword_unique: (_) => make_keyword("unique"),
    keyword_foreign: (_) => make_keyword("foreign"),
    keyword_references: (_) => make_keyword("references"),
    keyword_concurrently: (_) => make_keyword("concurrently"),
    keyword_btree: (_) => make_keyword("btree"),
    keyword_hash: (_) => make_keyword("hash"),
    keyword_gist: (_) => make_keyword("gist"),
    keyword_spgist: (_) => make_keyword("spgist"),
    keyword_gin: (_) => make_keyword("gin"),
    keyword_brin: (_) => make_keyword("brin"),
    keyword_like: (_) => choice(make_keyword("like"), make_keyword("ilike")),
    keyword_similar: (_) => make_keyword("similar"),
    keyword_unsigned: (_) => make_keyword("unsigned"),
    keyword_zerofill: (_) => make_keyword("zerofill"),
    keyword_conflict: (_) => make_keyword("conflict"),
    keyword_do: (_) => make_keyword("do"),
    keyword_nothing: (_) => make_keyword("nothing"),
    keyword_high_priority: (_) => make_keyword("high_priority"),
    keyword_low_priority: (_) => make_keyword("low_priority"),
    keyword_delayed: (_) => make_keyword("delayed"),
    keyword_recursive: (_) => make_keyword("recursive"),
    keyword_cascaded: (_) => make_keyword("cascaded"),
    keyword_local: (_) => make_keyword("local"),
    keyword_current_timestamp: (_) => make_keyword("current_timestamp"),
    keyword_check: (_) => make_keyword("check"),
    keyword_option: (_) => make_keyword("option"),
    keyword_vacuum: (_) => make_keyword("vacuum"),
    keyword_wait: (_) => make_keyword("wait"),
    keyword_nowait: (_) => make_keyword("nowait"),
    keyword_attribute: (_) => make_keyword("attribute"),
    keyword_authorization: (_) => make_keyword("authorization"),
    keyword_action: (_) => make_keyword("action"),
    keyword_extension: (_) => make_keyword("extension"),
    keyword_copy: (_) => make_keyword("copy"),
    keyword_stdin: (_) => make_keyword("stdin"),
    keyword_freeze: (_) => make_keyword("freeze"),
    keyword_escape: (_) => make_keyword("escape"),
    keyword_encoding: (_) => make_keyword("encoding"),
    keyword_force_quote: (_) => make_keyword("force_quote"),
    keyword_quote: (_) => make_keyword("quote"),
    keyword_force_null: (_) => make_keyword("force_null"),
    keyword_force_not_null: (_) => make_keyword("force_not_null"),
    keyword_header: (_) => make_keyword("header"),
    keyword_match: (_) => make_keyword("match"),
    keyword_program: (_) => make_keyword("program"),
    keyword_plain: (_) => make_keyword("plain"),
    keyword_extended: (_) => make_keyword("extended"),
    keyword_main: (_) => make_keyword("main"),
    keyword_storage: (_) => make_keyword("storage"),
    keyword_compression: (_) => make_keyword("compression"),

    keyword_overriding: () => make_keyword("overriding"),
    keyword_system: () => make_keyword("system"),
    keyword_policy: (_) => make_keyword("policy"),
    keyword_permissive: (_) => make_keyword("permissive"),
    keyword_restrictive: (_) => make_keyword("restrictive"),
    keyword_public: (_) => make_keyword("public"),
    keyword_current_role: (_) => make_keyword("current_role"),
    keyword_current_user: (_) => make_keyword("current_user"),
    keyword_session_user: (_) => make_keyword("session_user"),

    keyword_grant: (_) => make_keyword("grant"),
    keyword_revoke: (_) => make_keyword("revoke"),
    keyword_granted: (_) => make_keyword("granted"),
    keyword_privileges: (_) => make_keyword("privileges"),
    keyword_inherit: (_) => make_keyword("inherit"),
    keyword_maintain: (_) => make_keyword("maintain"),
    keyword_functions: (_) => make_keyword("functions"),
    keyword_routines: (_) => make_keyword("routines"),
    keyword_procedures: (_) => make_keyword("procedures"),

    keyword_trigger: (_) => make_keyword("trigger"),
    keyword_function: (_) => make_keyword("function"),
    keyword_returns: (_) => make_keyword("returns"),
    keyword_return: (_) => make_keyword("return"),
    keyword_setof: (_) => make_keyword("setof"),
    keyword_atomic: (_) => make_keyword("atomic"),
    keyword_declare: (_) => make_keyword("declare"),
    keyword_language: (_) => make_keyword("language"),
    keyword_immutable: (_) => make_keyword("immutable"),
    keyword_stable: (_) => make_keyword("stable"),
    keyword_volatile: (_) => make_keyword("volatile"),
    keyword_leakproof: (_) => make_keyword("leakproof"),
    keyword_parallel: (_) => make_keyword("parallel"),
    keyword_safe: (_) => make_keyword("safe"),
    keyword_unsafe: (_) => make_keyword("unsafe"),
    keyword_restricted: (_) => make_keyword("restricted"),
    keyword_called: (_) => make_keyword("called"),
    keyword_input: (_) => make_keyword("input"),
    keyword_strict: (_) => make_keyword("strict"),
    keyword_cost: (_) => make_keyword("cost"),
    keyword_support: (_) => make_keyword("support"),
    keyword_definer: (_) => make_keyword("definer"),
    keyword_invoker: (_) => make_keyword("invoker"),
    keyword_security: (_) => make_keyword("security"),
    keyword_version: (_) => make_keyword("version"),
    keyword_out: (_) => make_keyword("out"),
    keyword_inout: (_) => make_keyword("inout"),
    keyword_variadic: (_) => make_keyword("variadic"),
    keyword_ordinality: (_) => make_keyword("ordinality"),

    keyword_session: (_) => make_keyword("session"),
    keyword_isolation: (_) => make_keyword("isolation"),
    keyword_level: (_) => make_keyword("level"),
    keyword_serializable: (_) => make_keyword("serializable"),
    keyword_repeatable: (_) => make_keyword("repeatable"),
    keyword_read: (_) => make_keyword("read"),
    keyword_write: (_) => make_keyword("write"),
    keyword_committed: (_) => make_keyword("committed"),
    keyword_uncommitted: (_) => make_keyword("uncommitted"),
    keyword_deferrable: (_) => make_keyword("deferrable"),
    keyword_names: (_) => make_keyword("names"),
    keyword_zone: (_) => make_keyword("zone"),
    keyword_immediate: (_) => make_keyword("immediate"),
    keyword_deferred: (_) => make_keyword("deferred"),
    keyword_constraints: (_) => make_keyword("constraints"),
    keyword_snapshot: (_) => make_keyword("snapshot"),
    keyword_characteristics: (_) => make_keyword("characteristics"),
    keyword_follows: (_) => make_keyword("follows"),
    keyword_precedes: (_) => make_keyword("precedes"),
    keyword_each: (_) => make_keyword("each"),
    keyword_instead: (_) => make_keyword("instead"),
    keyword_of: (_) => make_keyword("of"),
    keyword_initially: (_) => make_keyword("initially"),
    keyword_old: (_) => make_keyword("old"),
    keyword_new: (_) => make_keyword("new"),
    keyword_referencing: (_) => make_keyword("referencing"),
    keyword_statement: (_) => make_keyword("statement"),
    keyword_execute: (_) => make_keyword("execute"),
    keyword_procedure: (_) => make_keyword("procedure"),
    keyword_routine: (_) => make_keyword("routine"),
    keyword_object_id: (_) => make_keyword("object_id"),

    // Hive Keywords
    keyword_external: (_) => make_keyword("external"),
    keyword_stored: (_) => make_keyword("stored"),
    keyword_virtual: (_) => make_keyword("virtual"),
    keyword_cached: (_) => make_keyword("cached"),
    keyword_uncached: (_) => make_keyword("uncached"),
    keyword_replication: (_) => make_keyword("replication"),
    keyword_tblproperties: (_) => make_keyword("tblproperties"),
    keyword_compute: (_) => make_keyword("compute"),
    keyword_stats: (_) => make_keyword("stats"),
    keyword_statistics: (_) => make_keyword("statistics"),
    keyword_optimize: (_) => make_keyword("optimize"),
    keyword_rewrite: (_) => make_keyword("rewrite"),
    keyword_bin_pack: (_) => make_keyword("bin_pack"),
    keyword_incremental: (_) => make_keyword("incremental"),
    keyword_location: (_) => make_keyword("location"),
    keyword_partitioned: (_) => make_keyword("partitioned"),
    keyword_comment: (_) => make_keyword("comment"),
    keyword_sort: (_) => make_keyword("sort"),
    keyword_format: (_) => make_keyword("format"),
    keyword_delimited: (_) => make_keyword("delimited"),
    keyword_delimiter: (_) => make_keyword("delimiter"),
    keyword_fields: (_) => make_keyword("fields"),
    keyword_terminated: (_) => make_keyword("terminated"),
    keyword_escaped: (_) => make_keyword("escaped"),
    keyword_lines: (_) => make_keyword("lines"),
    keyword_cache: (_) => make_keyword("cache"),
    keyword_metadata: (_) => make_keyword("metadata"),
    keyword_noscan: (_) => make_keyword("noscan"),

    // Hive file formats
    keyword_parquet: (_) => make_keyword("parquet"),
    keyword_rcfile: (_) => make_keyword("rcfile"),
    keyword_csv: (_) => make_keyword("csv"),
    keyword_textfile: (_) => make_keyword("textfile"),
    keyword_avro: (_) => make_keyword("avro"),
    keyword_sequencefile: (_) => make_keyword("sequencefile"),
    keyword_orc: (_) => make_keyword("orc"),
    keyword_jsonfile: (_) => make_keyword("jsonfile"),

    // Operators
    is_not: ($) => prec.left(seq($.keyword_is, $.keyword_not)),
    not_like: ($) => seq($.keyword_not, $.keyword_like),
    similar_to: ($) => seq($.keyword_similar, $.keyword_to),
    not_similar_to: ($) => seq($.keyword_not, $.keyword_similar, $.keyword_to),
    distinct_from: ($) => seq($.keyword_is, $.keyword_distinct, $.keyword_from),
    not_distinct_from: ($) =>
      seq($.keyword_is, $.keyword_not, $.keyword_distinct, $.keyword_from),

    _temporary: ($) => choice($.keyword_temp, $.keyword_temporary),
    _not_null: ($) => seq($.keyword_not, $.keyword_null),
    _primary_key: ($) => seq($.keyword_primary, $.keyword_key),
    _if_exists: ($) => seq($.keyword_if, $.keyword_exists),
    _if_not_exists: ($) => seq($.keyword_if, $.keyword_not, $.keyword_exists),
    _or_replace: ($) => seq($.keyword_or, $.keyword_replace),
    _default_null: ($) => seq($.keyword_default, $.keyword_null),
    _current_row: ($) => seq($.keyword_current, $.keyword_row),
    _exclude_current_row: ($) =>
      seq($.keyword_exclude, $.keyword_current, $.keyword_row),
    _exclude_group: ($) => seq($.keyword_exclude, $.keyword_group),
    _exclude_no_others: ($) =>
      seq($.keyword_exclude, $.keyword_no, $.keyword_others),
    _exclude_ties: ($) => seq($.keyword_exclude, $.keyword_ties),
    _check_option: ($) => seq($.keyword_check, $.keyword_option),
    direction: ($) => choice($.keyword_desc, $.keyword_asc),

    // Types
    keyword_null: (_) => make_keyword("null"),
    keyword_true: (_) => make_keyword("true"),
    keyword_false: (_) => make_keyword("false"),

    keyword_boolean: (_) => make_keyword("boolean"),
    keyword_bit: (_) => make_keyword("bit"),
    keyword_binary: (_) => make_keyword("binary"),
    keyword_varbinary: (_) => make_keyword("varbinary"),
    keyword_image: (_) => make_keyword("image"),

    keyword_smallserial: (_) =>
      choice(make_keyword("smallserial"), make_keyword("serial2")),
    keyword_serial: (_) =>
      choice(make_keyword("serial"), make_keyword("serial4")),
    keyword_bigserial: (_) =>
      choice(make_keyword("bigserial"), make_keyword("serial8")),
    keyword_tinyint: (_) =>
      choice(make_keyword("tinyint"), make_keyword("int1")),
    keyword_smallint: (_) =>
      choice(make_keyword("smallint"), make_keyword("int2")),
    keyword_mediumint: (_) =>
      choice(make_keyword("mediumint"), make_keyword("int3")),
    keyword_int: (_) =>
      choice(
        make_keyword("int"),
        make_keyword("integer"),
        make_keyword("int4")
      ),
    keyword_bigint: (_) => choice(make_keyword("bigint"), make_keyword("int8")),
    keyword_decimal: (_) => make_keyword("decimal"),
    keyword_numeric: (_) => make_keyword("numeric"),
    keyword_real: (_) => choice(make_keyword("real"), make_keyword("float4")),
    keyword_float: (_) => make_keyword("float"),
    keyword_double: (_) => make_keyword("double"),
    keyword_precision: (_) => make_keyword("precision"),
    keyword_inet: (_) => make_keyword("inet"),

    keyword_money: (_) => make_keyword("money"),
    keyword_smallmoney: (_) => make_keyword("smallmoney"),
    keyword_varying: (_) => make_keyword("varying"),

    keyword_char: (_) =>
      choice(make_keyword("char"), make_keyword("character")),
    keyword_nchar: (_) => make_keyword("nchar"),
    keyword_varchar: ($) =>
      choice(
        make_keyword("varchar"),
        seq(make_keyword("character"), $.keyword_varying)
      ),
    keyword_nvarchar: (_) => make_keyword("nvarchar"),
    keyword_text: (_) => make_keyword("text"),
    keyword_string: (_) => make_keyword("string"),
    keyword_uuid: (_) => make_keyword("uuid"),

    keyword_json: (_) => make_keyword("json"),
    keyword_jsonb: (_) => make_keyword("jsonb"),
    keyword_xml: (_) => make_keyword("xml"),

    keyword_bytea: (_) => make_keyword("bytea"),

    keyword_enum: (_) => make_keyword("enum"),

    keyword_date: (_) => make_keyword("date"),
    keyword_datetime: (_) => make_keyword("datetime"),
    keyword_datetime2: (_) => make_keyword("datetime2"),
    keyword_smalldatetime: (_) => make_keyword("smalldatetime"),
    keyword_datetimeoffset: (_) => make_keyword("datetimeoffset"),
    keyword_time: (_) => make_keyword("time"),
    keyword_timestamp: (_) => make_keyword("timestamp"),
    keyword_timestamptz: (_) => make_keyword("timestamptz"),
    keyword_interval: (_) => make_keyword("interval"),

    keyword_geometry: (_) => make_keyword("geometry"),
    keyword_geography: (_) => make_keyword("geography"),
    keyword_box2d: (_) => make_keyword("box2d"),
    keyword_box3d: (_) => make_keyword("box3d"),

    keyword_oid: (_) => make_keyword("oid"),
    keyword_oids: (_) => make_keyword("oids"),
    keyword_name: (_) => make_keyword("name"),
    keyword_regclass: (_) => make_keyword("regclass"),
    keyword_regnamespace: (_) => make_keyword("regnamespace"),
    keyword_regproc: (_) => make_keyword("regproc"),
    keyword_regtype: (_) => make_keyword("regtype"),

    keyword_array: (_) => make_keyword("array"), // not included in _type since it's a constructor literal

    type: ($) =>
      prec.left(
        seq(
          choice(
            $.keyword_boolean,
            $.bit,
            $.binary,
            $.varbinary,
            $.keyword_image,

            $.keyword_smallserial,
            $.keyword_serial,
            $.keyword_bigserial,

            $.tinyint,
            $.smallint,
            $.mediumint,
            $.int,
            $.bigint,
            $.decimal,
            $.numeric,
            $.double,
            $.float,

            $.keyword_money,
            $.keyword_smallmoney,

            $.char,
            $.varchar,
            $.nchar,
            $.nvarchar,
            $.numeric,
            $.keyword_string,
            $.keyword_text,

            $.keyword_uuid,

            $.keyword_json,
            $.keyword_jsonb,
            $.keyword_xml,

            $.keyword_bytea,
            $.keyword_inet,

            $.enum,

            $.keyword_date,
            $.keyword_datetime,
            $.keyword_datetime2,
            $.datetimeoffset,
            $.keyword_smalldatetime,
            $.time,
            $.timestamp,
            $.keyword_timestamptz,
            $.keyword_interval,

            $.keyword_geometry,
            $.keyword_geography,
            $.keyword_box2d,
            $.keyword_box3d,

            $.keyword_oid,
            $.keyword_name,
            $.keyword_regclass,
            $.keyword_regnamespace,
            $.keyword_regproc,
            $.keyword_regtype,

            field("custom_type", $.object_reference)
          ),
          optional($.array_size_definition)
        )
      ),

    array_size_definition: ($) =>
      prec.left(
        choice(
          seq($.keyword_array, optional($._array_size_definition)),
          repeat1($._array_size_definition)
        )
      ),

    _array_size_definition: ($) =>
      seq("[", optional(field("size", alias($._integer, $.literal))), "]"),

    tinyint: ($) => unsigned_type($, parametric_type($, $.keyword_tinyint)),
    smallint: ($) => unsigned_type($, parametric_type($, $.keyword_smallint)),
    mediumint: ($) => unsigned_type($, parametric_type($, $.keyword_mediumint)),
    int: ($) => unsigned_type($, parametric_type($, $.keyword_int)),
    bigint: ($) => unsigned_type($, parametric_type($, $.keyword_bigint)),

    bit: ($) =>
      choice(
        $.keyword_bit,
        seq(
          $.keyword_bit,
          prec(0, parametric_type($, $.keyword_varying, ["precision"]))
        ),
        prec(1, parametric_type($, $.keyword_bit, ["precision"]))
      ),

    binary: ($) => parametric_type($, $.keyword_binary, ["precision"]),
    varbinary: ($) => parametric_type($, $.keyword_varbinary, ["precision"]),

    // TODO: should qualify against /\\b(0?[1-9]|[1-4][0-9]|5[0-4])\\b/g
    float: ($) =>
      choice(
        parametric_type($, $.keyword_float, ["precision"]),
        unsigned_type(
          $,
          parametric_type($, $.keyword_float, ["precision", "scale"])
        )
      ),

    double: ($) =>
      choice(
        make_keyword("float8"),
        unsigned_type(
          $,
          parametric_type($, $.keyword_double, ["precision", "scale"])
        ),
        unsigned_type(
          $,
          parametric_type($, seq($.keyword_double, $.keyword_precision), [
            "precision",
            "scale",
          ])
        ),
        unsigned_type(
          $,
          parametric_type($, $.keyword_real, ["precision", "scale"])
        )
      ),

    decimal: ($) =>
      choice(
        parametric_type($, $.keyword_decimal, ["precision"]),
        parametric_type($, $.keyword_decimal, ["precision", "scale"])
      ),
    numeric: ($) =>
      choice(
        parametric_type($, $.keyword_numeric, ["precision"]),
        parametric_type($, $.keyword_numeric, ["precision", "scale"])
      ),
    char: ($) => parametric_type($, $.keyword_char),
    varchar: ($) => parametric_type($, $.keyword_varchar),
    nchar: ($) => parametric_type($, $.keyword_nchar),
    nvarchar: ($) => parametric_type($, $.keyword_nvarchar),

    _include_time_zone: ($) =>
      seq(
        choice($.keyword_with, $.keyword_without),
        $.keyword_time,
        $.keyword_zone
      ),
    datetimeoffset: ($) => parametric_type($, $.keyword_datetimeoffset),
    time: ($) =>
      seq(parametric_type($, $.keyword_time), optional($._include_time_zone)),
    timestamp: ($) =>
      seq(
        parametric_type($, $.keyword_timestamp),
        optional($._include_time_zone)
      ),
    timestamptz: ($) => parametric_type($, $.keyword_timestamptz),

    enum: ($) =>
      seq(
        $.keyword_enum,
        paren_list(field("value", alias($._literal_string, $.literal)), true)
      ),

    array: ($) =>
      seq(
        $.keyword_array,
        choice(
          seq("[", comma_list($._expression, false), "]"),
          seq("(", $._dml_read, ")")
        )
      ),

    comment: (_) => /--.*/,
    // https://stackoverflow.com/questions/13014947/regex-to-match-a-c-style-multiline-comment
    marginalia: (_) => /\/\*[^*]*\*+(?:[^/*][^*]*\*+)*\//,

    transaction: ($) =>
      seq(
        $.keyword_begin,
        optional($.keyword_transaction),
        optional(";"),
        repeat(seq($.statement, ";")),
        choice($._commit, $._rollback)
      ),

    _commit: ($) => seq($.keyword_commit, optional($.keyword_transaction)),

    _rollback: ($) => seq($.keyword_rollback, optional($.keyword_transaction)),

    block: ($) =>
      seq(
        $.keyword_begin,
        optional(";"),
        repeat(seq($.statement, ";")),
        $.keyword_end
      ),

    statement: ($) =>
      seq(
        optional(
          seq(
            $.keyword_explain,
            optional($.keyword_analyze),
            optional($.keyword_verbose)
          )
        ),
        choice(
          $._ddl_statement,
          $._dml_write,
          optional_parenthesis($._dml_read)
        )
      ),

    _ddl_statement: ($) =>
      choice(
        $._create_statement,
        $._alter_statement,
        $._drop_statement,
        $._rename_statement,
        $._optimize_statement,
        $._merge_statement,
        $.comment_statement,
        $.set_statement,
        $.reset_statement,
        $.revoke_statement,
        $.grant_statement
      ),

    _cte: ($) =>
      seq(
        $.keyword_with,
        optional($.keyword_recursive),
        $.cte,
        repeat(seq(",", $.cte))
      ),

    _dml_write: ($) =>
      seq(
        seq(
          optional($._cte),
          choice(
            $._delete_statement,
            $._insert_statement,
            $._update_statement,
            $._truncate_statement,
            $._copy_statement
          )
        )
      ),

    _dml_read: ($) =>
      seq(
        optional(optional_parenthesis($._cte)),
        optional_parenthesis(
          choice(
            $._select_statement,
            $.set_operation,
            $._show_statement,
            $._unload_statement
          )
        )
      ),

    // athena
    _unload_statement: ($) =>
      seq(
        $.keyword_unload,
        wrapped_in_parenthesis($._select_statement),
        $.keyword_to,
        $._single_quote_string,
        $.storage_parameters
      ),

    _show_statement: ($) =>
      seq($.keyword_show, choice($._show_create, $.keyword_all)),

    _show_create: ($) =>
      seq(
        $.keyword_create,
        choice(
          // Trino/Presto/MySQL
          $.keyword_schema,
          $.keyword_table,
          seq(optional($.keyword_materialized), $.keyword_view),
          // MySQL
          $.keyword_user,
          $.keyword_trigger,
          $.keyword_procedure,
          $.keyword_function
        ),
        $.object_reference
      ),

    cte: ($) =>
      seq(
        $.any_identifier,
        optional(paren_list(field("argument", $.any_identifier), false)),
        $.keyword_as,
        optional(seq(optional($.keyword_not), $.keyword_materialized)),
        wrapped_in_parenthesis(
          alias(choice($._dml_read, $._dml_write), $.statement)
        )
      ),

    set_operation: ($) =>
      seq(
        $._select_statement,
        repeat1(
          seq(
            field(
              "operation",
              choice(
                seq($.keyword_union, optional($.keyword_all)),
                $.keyword_except,
                $.keyword_intersect
              )
            ),
            $._select_statement
          )
        )
      ),

    _select_statement: ($) =>
      optional_parenthesis(
        seq(
          $.select,
          optional(seq($.keyword_into, $.select_expression)),
          optional($.from)
        )
      ),

    comment_statement: ($) =>
      seq(
        $.keyword_comment,
        $.keyword_on,
        $._comment_target,
        $.keyword_is,
        choice($.keyword_null, alias($._literal_string, $.literal))
      ),

    _argmode: ($) =>
      choice(
        $.keyword_in,
        $.keyword_out,
        $.keyword_inout,
        $.keyword_variadic,
        seq($.keyword_in, $.keyword_out)
      ),

    function_argument: ($) =>
      seq(
        optional($._argmode),
        optional($.any_identifier),
        $.type,
        optional(seq(choice($.keyword_default, "="), $.literal))
      ),

    function_arguments: ($) => paren_list($.function_argument, false),

    _comment_target: ($) =>
      choice(
        // TODO: access method
        // TODO: aggregate
        $.cast,
        // TODO: collation
        seq($.keyword_column, alias($._qualified_field, $.object_reference)),
        // TODO: constraint (on domain)
        // TODO: conversion
        seq($.keyword_database, $.any_identifier),
        // TODO: domain
        seq($.keyword_extension, $.object_reference),
        // TODO: event trigger
        // TODO: foreign data wrapper
        // TODO: foreign table
        seq(
          $.keyword_function,
          $.object_reference,
          optional($.function_arguments)
        ),
        seq($.keyword_index, $.object_reference),
        // TODO: large object
        seq($.keyword_materialized, $.keyword_view, $.object_reference),
        // TODO: operator (|class|family)
        // TODO: policy
        // TODO: (procedural) language
        // TODO: procedure
        // TODO: publication
        seq($.keyword_role, $.any_identifier),
        // TODO: routine
        // TODO: rule
        seq($.keyword_schema, $.any_identifier),
        seq($.keyword_sequence, $.object_reference),
        // TODO: server
        // TODO: statistics
        // TODO: subscription
        seq($.keyword_table, $.object_reference),
        seq($.keyword_tablespace, $.any_identifier),
        // TODO: text search (configuration|dictionary|parser|template)
        // TODO: transform for
        seq(
          $.keyword_trigger,
          $.any_identifier,
          $.keyword_on,
          $.object_reference
        ),
        seq($.keyword_type, $.any_identifier),
        seq($.keyword_view, $.object_reference)
      ),

    select: ($) =>
      seq(
        $.keyword_select,
        seq(optional($.keyword_distinct), $.select_expression)
      ),

    select_expression: ($) => seq($.term, repeat(seq(",", $.term))),

    term: ($) =>
      seq(
        field("value", choice($.all_fields, $._expression)),
        optional($._alias)
      ),

    _truncate_statement: ($) =>
      seq(
        $.keyword_truncate,
        optional($.keyword_table),
        optional($.keyword_only),
        comma_list($.object_reference, false),
        optional($._drop_behavior)
      ),

    _delete_statement: ($) =>
      seq($.delete, alias($._delete_from, $.from), optional($.returning)),

    _delete_from: ($) =>
      seq(
        $.keyword_from,
        optional($.keyword_only),
        $.object_reference,
        optional($.where),
        optional($.order_by),
        optional($.limit)
      ),

    delete: ($) => seq($.keyword_delete, optional($.index_hint)),

    _create_statement: ($) =>
      seq(
        choice(
          $.create_table,
          $.create_view,
          $.create_materialized_view,
          $.create_index,
          $.create_function,
          $.create_type,
          $.create_database,
          $.create_role,
          $.create_sequence,
          $.create_extension,
          $.create_trigger,
          $.create_policy,
          prec.left(seq($.create_schema, repeat($._create_statement)))
        )
      ),

    _table_settings: ($) =>
      choice(
        $.table_partition,
        $.stored_as,
        $.storage_location,
        $.row_format,
        seq($.keyword_tblproperties, paren_list($.table_option, true)),
        seq($.keyword_without, $.keyword_oids),
        $.storage_parameters,
        $.table_option
      ),

    storage_parameters: ($) =>
      seq(
        $.keyword_with,
        paren_list(
          seq($.any_identifier, optional(seq("=", choice($.literal, $.array)))),
          true
        )
      ),

    // left precedence because 'quoted' table options otherwise conflict with
    // `create function` string bodies; if you remove this precedence you will
    // have to also disable the `_literal_string` choice for the `name` field
    // in =-assigned `table_option`s
    create_table: ($) =>
      prec.left(
        seq(
          $.keyword_create,
          optional(
            choice($._temporary, $.keyword_unlogged, $.keyword_external)
          ),
          $.keyword_table,
          optional($._if_not_exists),
          $.object_reference,
          choice(
            seq(
              $.column_definitions,
              repeat($._table_settings),
              optional(seq($.keyword_as, $._select_statement))
            ),
            seq(repeat($._table_settings), seq($.keyword_as, $.create_query))
          )
        )
      ),

    create_policy: ($) =>
      seq(
        $.keyword_create,
        $.keyword_policy,
        $.any_identifier,
        $.keyword_on,
        $.object_reference,
        optional(
          seq($.keyword_as, choice($.keyword_permissive, $.keyword_restrictive))
        ),
        optional(
          seq(
            $.keyword_for,
            choice(
              $.keyword_all,
              $.keyword_select,
              $.keyword_insert,
              $.keyword_update,
              $.keyword_delete
            )
          )
        ),
        optional($.policy_to_role),
        optional($.check_or_using_clause)
      ),

    alter_policy: ($) =>
      seq(
        seq($.keyword_alter, $.keyword_policy, $.any_identifier),
        optional(
          seq(
            $.keyword_on,
            $.object_reference,
            choice(
              seq($.keyword_rename, $.keyword_to, $.any_identifier),
              $.policy_to_role,
              optional($.check_or_using_clause)
            )
          )
        )
      ),

    policy_to_role: ($) =>
      seq(
        $.keyword_to,
        comma_list(
          choice(
            $.any_identifier,
            $.keyword_public,
            $.keyword_current_user,
            $.keyword_current_role,
            $.keyword_session_user
          ),
          true
        )
      ),

    drop_policy: ($) =>
      seq(
        seq(
          $.keyword_drop,
          $.keyword_policy,
          optional($._if_exists),
          $.any_identifier
        ),
        optional(
          seq(
            $.keyword_on,
            $.object_reference,
            optional(choice($.keyword_cascade, $.keyword_restrict))
          )
        )
      ),

    check_or_using_clause: ($) =>
      choice(
        seq($.keyword_using, wrapped_in_parenthesis($._expression)),
        seq(
          $.keyword_with,
          $.keyword_check,
          wrapped_in_parenthesis($._expression)
        )
      ),

    reset_statement: ($) =>
      seq(
        $.keyword_reset,
        choice(
          $.object_reference,
          $.keyword_all,
          seq($.keyword_session, $.keyword_authorization),
          $.keyword_role
        )
      ),

    _transaction_mode: ($) =>
      seq(
        $.keyword_isolation,
        $.keyword_level,
        choice(
          $.keyword_serializable,
          seq($.keyword_repeatable, $.keyword_read),
          seq($.keyword_read, $.keyword_committed),
          seq($.keyword_read, $.keyword_uncommitted)
        ),
        choice(
          seq($.keyword_read, $.keyword_write),
          seq($.keyword_read, $.keyword_only)
        ),
        optional($.keyword_not),
        $.keyword_deferrable
      ),

    set_statement: ($) =>
      seq(
        $.keyword_set,
        choice(
          seq(
            optional(choice($.keyword_session, $.keyword_local)),
            choice(
              seq(
                $.object_reference,
                choice($.keyword_to, "="),
                choice(
                  $.literal,
                  $.keyword_default,
                  $.any_identifier,
                  $.keyword_on,
                  $.keyword_off
                )
              ),
              seq($.keyword_schema, $.literal),
              seq($.keyword_names, $.literal),
              seq(
                $.keyword_time,
                $.keyword_zone,
                choice($.literal, $.keyword_local, $.keyword_default)
              ),
              seq(
                $.keyword_session,
                $.keyword_authorization,
                choice($.any_identifier, $.keyword_default)
              ),
              seq($.keyword_role, choice($.any_identifier, $.keyword_none))
            )
          ),
          seq(
            $.keyword_constraints,
            choice($.keyword_all, comma_list($.any_identifier, true)),
            choice($.keyword_deferred, $.keyword_immediate)
          ),
          seq($.keyword_transaction, $._transaction_mode),
          seq($.keyword_transaction, $.keyword_snapshot, $._transaction_mode),
          seq(
            $.keyword_session,
            $.keyword_characteristics,
            $.keyword_as,
            $.keyword_transaction,
            $._transaction_mode
          )
        )
      ),

    create_query: ($) => $._dml_read,

    create_view: ($) =>
      prec.right(
        seq(
          $.keyword_create,
          optional($._or_replace),
          optional($._temporary),
          optional($.keyword_recursive),
          $.keyword_view,
          optional($._if_not_exists),
          $.object_reference,
          optional(paren_list($.any_identifier, false)),
          $.keyword_as,
          $.create_query,
          optional(
            seq(
              $.keyword_with,
              optional(choice($.keyword_local, $.keyword_cascaded)),
              $._check_option
            )
          )
        )
      ),

    create_materialized_view: ($) =>
      prec.right(
        seq(
          $.keyword_create,
          optional($._or_replace),
          $.keyword_materialized,
          $.keyword_view,
          optional($._if_not_exists),
          $.object_reference,
          $.keyword_as,
          $.create_query,
          optional(
            choice(
              seq($.keyword_with, $.keyword_data),
              seq($.keyword_with, $.keyword_no, $.keyword_data)
            )
          )
        )
      ),

    // This is only used in create function statement, it is not needed to check
    // the start tag match the end one. The usage of this syntax in other
    // context is done by _dollar_string.
    dollar_quote: () => /\$[^\$]*\$/,

    create_function: ($) =>
      seq(
        $.keyword_create,
        optional($._or_replace),
        $.keyword_function,
        $.object_reference,
        $.function_arguments,
        $.keyword_returns,
        choice(
          $.type,
          seq($.keyword_setof, $.type),
          seq($.keyword_table, $.column_definitions),
          $.keyword_trigger
        ),
        repeat(
          choice(
            $.function_language,
            $.function_volatility,
            $.function_leakproof,
            $.function_security,
            $.function_safety,
            $.function_strictness,
            $.function_cost,
            $.function_rows,
            $.function_support
          )
        ),
        // ensure that there's only one function body -- other specifiers are less
        // variable but the body can have all manner of conflicting stuff
        $.function_body,
        repeat(
          choice(
            $.function_language,
            $.function_volatility,
            $.function_leakproof,
            $.function_security,
            $.function_safety,
            $.function_strictness,
            $.function_cost,
            $.function_rows,
            $.function_support
          )
        )
      ),

    _function_return: ($) => seq($.keyword_return, $._expression),

    function_declaration: ($) =>
      seq(
        $.any_identifier,
        $.type,
        optional(
          seq(
            ":=",
            choice(
              wrapped_in_parenthesis($.statement),
              // TODO are there more possibilities here? We can't use `_expression` since
              // that includes subqueries
              $.literal
            )
          )
        ),
        ";"
      ),

    _function_body_statement: ($) => choice($.statement, $._function_return),

    function_body: ($) =>
      choice(
        seq($._function_return, ";"),
        seq(
          $.keyword_begin,
          $.keyword_atomic,
          repeat1(seq($._function_body_statement, ";")),
          $.keyword_end
        ),
        seq(
          $.keyword_as,
          alias($._dollar_quoted_string_start_tag, $.dollar_quote),
          optional(seq($.keyword_declare, repeat1($.function_declaration))),
          $.keyword_begin,
          repeat1(seq($._function_body_statement, ";")),
          $.keyword_end,
          optional(";"),
          alias($._dollar_quoted_string_end_tag, $.dollar_quote)
        ),
        seq(
          $.keyword_as,
          alias(
            choice($._single_quote_string, $._double_quote_string),
            $.literal
          )
        ),
        seq(
          $.keyword_as,
          alias($._dollar_quoted_string_start_tag, $.dollar_quote),
          $._function_body_statement,
          optional(";"),
          alias($._dollar_quoted_string_end_tag, $.dollar_quote)
        )
      ),

    function_language: ($) =>
      seq(
        $.keyword_language,
        // TODO Maybe we should do different version of function_body_statement in
        // regard to the defined language to match either sql, plsql or
        // plpgsql. Currently the function_body_statement support only sql.  And
        // maybe for other language the function_body should be a string.
        $.any_identifier
      ),

    function_volatility: ($) =>
      choice($.keyword_immutable, $.keyword_stable, $.keyword_volatile),

    function_leakproof: ($) =>
      seq(optional($.keyword_not), $.keyword_leakproof),

    function_security: ($) =>
      seq(
        optional($.keyword_external),
        $.keyword_security,
        choice($.keyword_invoker, $.keyword_definer)
      ),

    function_safety: ($) =>
      seq(
        $.keyword_parallel,
        choice($.keyword_safe, $.keyword_unsafe, $.keyword_restricted)
      ),

    function_strictness: ($) =>
      choice(
        seq(
          choice($.keyword_called, seq($.keyword_returns, $.keyword_null)),
          $.keyword_on,
          $.keyword_null,
          $.keyword_input
        ),
        $.keyword_strict
      ),

    function_cost: ($) => seq($.keyword_cost, $._natural_number),

    function_rows: ($) => seq($.keyword_rows, $._natural_number),

    function_support: ($) =>
      seq($.keyword_support, alias($._literal_string, $.literal)),

    _operator_class: ($) =>
      seq(
        field("opclass", $.any_identifier),
        optional(
          field(
            "opclass_parameters",
            wrapped_in_parenthesis(comma_list($.term, false))
          )
        )
      ),

    _index_field: ($) =>
      seq(
        choice(
          field("expression", wrapped_in_parenthesis($._expression)),
          field("function", $.invocation),
          field("column", $._column)
        ),
        optional(seq($.keyword_collate, $.any_identifier)),
        optional($._operator_class),
        optional($.direction),
        optional(seq($.keyword_nulls, choice($.keyword_first, $.keyword_last)))
      ),

    index_fields: ($) =>
      wrapped_in_parenthesis(comma_list(alias($._index_field, $.field), false)),

    create_index: ($) =>
      seq(
        $.keyword_create,
        optional($.keyword_unique),
        $.keyword_index,
        optional($.keyword_concurrently),
        optional(seq(optional($._if_not_exists), field("column", $._column))),
        $.keyword_on,
        optional($.keyword_only),
        seq(
          $.object_reference,
          optional(
            seq(
              $.keyword_using,
              choice(
                $.keyword_btree,
                $.keyword_hash,
                $.keyword_gist,
                $.keyword_spgist,
                $.keyword_gin,
                $.keyword_brin
              )
            )
          ),
          $.index_fields
        ),
        optional($.where)
      ),

    create_schema: ($) =>
      prec.left(
        seq(
          $.keyword_create,
          $.keyword_schema,
          choice(
            seq(
              optional($._if_not_exists),
              $.any_identifier,
              optional(seq($.keyword_authorization, $.any_identifier))
            ),
            seq($.keyword_authorization, $.any_identifier)
          )
        )
      ),

    _with_settings: ($) =>
      seq(
        field("name", $.any_identifier),
        optional("="),
        field(
          "value",
          choice($.any_identifier, alias($._single_quote_string, $.literal))
        )
      ),

    create_database: ($) =>
      seq(
        $.keyword_create,
        $.keyword_database,
        optional($._if_not_exists),
        $.any_identifier,
        optional($.keyword_with),
        repeat($._with_settings)
      ),

    create_role: ($) =>
      seq(
        $.keyword_create,
        choice($.keyword_user, $.keyword_role, $.keyword_group),
        $.any_identifier,
        optional($.keyword_with),
        repeat(choice($._user_access_role_config, $._role_options))
      ),

    _role_options: ($) =>
      choice(
        field("option", $.any_identifier),
        seq(
          $.keyword_valid,
          $.keyword_until,
          field("valid_until", alias($._literal_string, $.literal))
        ),
        seq(
          $.keyword_connection,
          $.keyword_limit,
          field("connection_limit", alias($._integer, $.literal))
        ),
        seq(
          optional($.keyword_encrypted),
          $.keyword_password,
          choice(
            field("password", alias($._literal_string, $.literal)),
            $.keyword_null
          )
        )
      ),

    _user_access_role_config: ($) =>
      seq(
        choice(
          seq(optional($.keyword_in), $.keyword_role),
          seq($.keyword_in, $.keyword_group),
          $.keyword_admin,
          $.keyword_user
        ),
        comma_list($.any_identifier, true)
      ),

    create_sequence: ($) =>
      seq(
        $.keyword_create,
        optional(
          choice(
            choice($.keyword_temporary, $.keyword_temp),
            $.keyword_unlogged
          )
        ),
        $.keyword_sequence,
        optional($._if_not_exists),
        $.object_reference,
        repeat(
          choice(
            seq($.keyword_as, $.type),
            seq(
              $.keyword_increment,
              optional($.keyword_by),
              field("increment", alias($._integer, $.literal))
            ),
            seq(
              $.keyword_minvalue,
              choice($.literal, seq($.keyword_no, $.keyword_minvalue))
            ),
            seq($.keyword_no, $.keyword_minvalue),
            seq(
              $.keyword_maxvalue,
              choice($.literal, seq($.keyword_no, $.keyword_maxvalue))
            ),
            seq($.keyword_no, $.keyword_maxvalue),
            seq(
              $.keyword_start,
              optional($.keyword_with),
              field("start", alias($._integer, $.literal))
            ),
            seq($.keyword_cache, field("cache", alias($._integer, $.literal))),
            seq(optional($.keyword_no), $.keyword_cycle),
            seq(
              $.keyword_owned,
              $.keyword_by,
              choice($.keyword_none, $.object_reference)
            )
          )
        )
      ),

    create_extension: ($) =>
      seq(
        $.keyword_create,
        $.keyword_extension,
        optional($._if_not_exists),
        $.any_identifier,
        optional($.keyword_with),
        optional(seq($.keyword_schema, $.any_identifier)),
        optional(
          seq(
            $.keyword_version,
            choice($.any_identifier, alias($._literal_string, $.literal))
          )
        ),
        optional($.keyword_cascade)
      ),

    create_trigger: ($) =>
      seq(
        $.keyword_create,
        optional($._or_replace),
        // mariadb
        optional(seq($.keyword_definer, "=", $.any_identifier)),
        optional($.keyword_constraint),
        // sqlite
        optional($._temporary),
        $.keyword_trigger,
        // sqlite/mariadb
        optional($._if_not_exists),
        $.object_reference,
        choice(
          $.keyword_before,
          $.keyword_after,
          seq($.keyword_instead, $.keyword_of)
        ),
        $._create_trigger_event,
        repeat(seq($.keyword_or, $._create_trigger_event)),
        $.keyword_on,
        $.object_reference,
        repeat(
          choice(
            seq($.keyword_from, $.object_reference),
            choice(
              seq($.keyword_not, $.keyword_deferrable),
              $.keyword_deferrable,
              seq($.keyword_initially, $.keyword_immediate),
              seq($.keyword_initially, $.keyword_deferred)
            ),
            seq(
              $.keyword_referencing,
              choice($.keyword_old, $.keyword_new),
              $.keyword_table,
              optional($.keyword_as),
              $.any_identifier
            ),
            seq(
              $.keyword_for,
              optional($.keyword_each),
              choice($.keyword_row, $.keyword_statement),
              // mariadb
              optional(
                seq(
                  choice($.keyword_follows, $.keyword_precedes),
                  $.any_identifier
                )
              )
            ),
            seq($.keyword_when, wrapped_in_parenthesis($._expression))
          )
        ),
        $.keyword_execute,
        choice($.keyword_function, $.keyword_procedure),
        $.object_reference,
        paren_list(field("parameter", $.term), false)
      ),

    _create_trigger_event: ($) =>
      choice(
        $.keyword_insert,
        seq(
          $.keyword_update,
          optional(seq($.keyword_of, comma_list($.column_identifier, true)))
        ),
        $.keyword_delete,
        $.keyword_truncate
      ),

    create_type: ($) =>
      seq(
        $.keyword_create,
        $.keyword_type,
        $.object_reference,
        optional(
          seq(
            choice(
              seq(
                $.keyword_as,
                $.column_definitions,
                optional(seq($.keyword_collate, $.any_identifier))
              ),
              seq($.keyword_as, $.keyword_enum, $.enum_elements),
              seq(
                optional(seq($.keyword_as, $.keyword_range)),
                paren_list($._with_settings, false)
              )
            )
          )
        )
      ),

    enum_elements: ($) =>
      seq(
        paren_list(
          field("enum_element", alias($._literal_string, $.literal)),
          false
        )
      ),

    _alter_statement: ($) =>
      seq(
        choice(
          $.alter_table,
          $.alter_view,
          $.alter_schema,
          $.alter_type,
          $.alter_index,
          $.alter_database,
          $.alter_role,
          $.alter_sequence,
          $.alter_policy
        )
      ),

    _rename_statement: ($) =>
      seq(
        $.keyword_rename,
        choice($.keyword_table, $.keyword_tables),
        optional($._if_exists),
        $.object_reference,
        optional(
          choice(
            $.keyword_nowait,
            seq(
              $.keyword_wait,
              field("timeout", alias($._natural_number, $.literal))
            )
          )
        ),
        $.keyword_to,
        $.object_reference,
        repeat(seq(",", $._rename_table_names))
      ),

    _rename_table_names: ($) =>
      seq($.object_reference, $.keyword_to, $.object_reference),

    alter_table: ($) =>
      seq(
        $.keyword_alter,
        $.keyword_table,
        optional($._if_exists),
        optional($.keyword_only),
        $.object_reference,
        choice(
          seq(
            $._alter_specifications,
            repeat(seq(",", $._alter_specifications))
          )
        )
      ),

    _alter_specifications: ($) =>
      choice(
        $.add_column,
        $.add_constraint,
        $.drop_constraint,
        $.alter_column,
        $.drop_column,
        $.rename_object,
        $.rename_column,
        $.set_schema,
        $.change_ownership
      ),

    // TODO: optional `keyword_add` is necessary to allow for chained alter statements in t-sql
    // maybe needs refactoring
    add_column: ($) =>
      seq(
        optional($.keyword_add),
        optional($.keyword_column),
        optional($._if_not_exists),
        $.column_definition,
        optional($.column_position)
      ),

    add_constraint: ($) =>
      seq(
        $.keyword_add,
        optional($.keyword_constraint),
        $.any_identifier,
        $.constraint
      ),

    drop_constraint: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_constraint,
        optional($._if_exists),
        $.any_identifier,
        optional($._drop_behavior)
      ),

    alter_column: ($) =>
      seq(
        // TODO constraint management
        $.keyword_alter,
        optional($.keyword_column),
        $.column_identifier,
        choice(
          seq(
            choice($.keyword_set, $.keyword_drop),
            $.keyword_not,
            $.keyword_null
          ),
          seq(
            optional(seq($.keyword_set, $.keyword_data)),
            $.keyword_type,
            field("type", $.type)
          ),
          seq(
            $.keyword_set,
            choice(
              seq($.keyword_statistics, field("statistics", $._integer)),
              seq(
                $.keyword_storage,
                choice(
                  $.keyword_plain,
                  $.keyword_external,
                  $.keyword_extended,
                  $.keyword_main,
                  $.keyword_default
                )
              ),
              seq(
                $.keyword_compression,
                field("compression_method", $._identifier)
              ),
              seq(paren_list($._key_value_pair, true)),
              seq($.keyword_default, $._expression)
            )
          ),
          seq($.keyword_drop, $.keyword_default)
        )
      ),

    column_position: ($) =>
      choice($.keyword_first, seq($.keyword_after, $.column_identifier)),

    drop_column: ($) =>
      seq(
        $.keyword_drop,
        optional($.keyword_column),
        optional($._if_exists),
        $.column_identifier
      ),

    rename_column: ($) =>
      seq(
        $.keyword_rename,
        optional($.keyword_column),
        $.column_identifier,
        $.keyword_to,
        field("new_name", $.any_identifier)
      ),

    alter_view: ($) =>
      seq(
        $.keyword_alter,
        $.keyword_view,
        optional($._if_exists),
        $.object_reference,
        choice(
          // TODO Postgres allows a single "alter column" to set or drop default
          $.rename_object,
          $.rename_column,
          $.set_schema,
          $.change_ownership
        )
      ),

    alter_schema: ($) =>
      seq(
        $.keyword_alter,
        $.keyword_schema,
        $.any_identifier,
        choice($.keyword_rename, $.keyword_owner),
        $.keyword_to,
        $.any_identifier
      ),

    alter_database: ($) =>
      seq(
        $.keyword_alter,
        $.keyword_database,
        $.any_identifier,
        optional($.keyword_with),
        choice(
          seq($.rename_object),
          seq($.change_ownership),
          seq(
            $.keyword_reset,
            choice(
              $.keyword_all,
              field("configuration_parameter", $.any_identifier)
            )
          ),
          seq(
            $.keyword_set,
            choice(
              seq($.keyword_tablespace, $.any_identifier),
              $.set_configuration
            )
          )
        )
      ),

    alter_role: ($) =>
      seq(
        $.keyword_alter,
        choice($.keyword_role, $.keyword_group, $.keyword_user),
        choice($.any_identifier, $.keyword_all),
        choice(
          $.rename_object,
          seq(optional($.keyword_with), repeat($._role_options)),
          seq(
            optional(seq($.keyword_in, $.keyword_database, $.any_identifier)),
            choice(
              seq($.keyword_set, $.set_configuration),
              seq(
                $.keyword_reset,
                choice($.keyword_all, field("option", $.any_identifier))
              )
            )
          )
        )
      ),

    set_configuration: ($) =>
      seq(
        field("option", $.any_identifier),
        choice(
          seq($.keyword_from, $.keyword_current),
          seq(
            choice($.keyword_to, "="),
            choice(
              field("parameter", $.any_identifier),
              $.literal,
              $.keyword_default
            )
          )
        )
      ),

    alter_index: ($) =>
      seq(
        $.keyword_alter,
        $.keyword_index,
        optional($._if_exists),
        $.any_identifier,
        choice(
          $.rename_object,
          seq(
            $.keyword_alter,
            optional($.keyword_column),
            alias($._natural_number, $.literal),
            $.keyword_set,
            $.keyword_statistics,
            alias($._natural_number, $.literal)
          ),
          seq($.keyword_reset, paren_list($.any_identifier, false)),
          seq(
            $.keyword_set,
            choice(
              seq($.keyword_tablespace, $.any_identifier),
              paren_list(
                seq($.any_identifier, "=", field("value", $.literal)),
                false
              )
            )
          )
        )
      ),

    alter_sequence: ($) =>
      seq(
        $.keyword_alter,
        $.keyword_sequence,
        optional($._if_exists),
        $.object_reference,
        choice(
          repeat1(
            choice(
              seq($.keyword_as, $.type),
              seq($.keyword_increment, optional($.keyword_by), $.literal),
              seq(
                $.keyword_minvalue,
                choice($.literal, seq($.keyword_no, $.keyword_minvalue))
              ),
              seq(
                $.keyword_maxvalue,
                choice($.literal, seq($.keyword_no, $.keyword_maxvalue))
              ),
              seq(
                $.keyword_start,
                optional($.keyword_with),
                field("start", alias($._integer, $.literal))
              ),
              seq(
                $.keyword_restart,
                optional($.keyword_with),
                field("restart", alias($._integer, $.literal))
              ),
              seq(
                $.keyword_cache,
                field("cache", alias($._integer, $.literal))
              ),
              seq(optional($.keyword_no), $.keyword_cycle),
              seq(
                $.keyword_owned,
                $.keyword_by,
                choice($.keyword_none, $.object_reference)
              )
            )
          ),
          $.rename_object,
          $.change_ownership,
          seq(
            $.keyword_set,
            choice(
              choice($.keyword_logged, $.keyword_unlogged),
              seq($.keyword_schema, $.any_identifier)
            )
          )
        )
      ),

    alter_type: ($) =>
      seq(
        $.keyword_alter,
        $.keyword_type,
        $.any_identifier,
        choice(
          $.change_ownership,
          $.set_schema,
          $.rename_object,
          seq(
            $.keyword_rename,
            $.keyword_attribute,
            $.any_identifier,
            $.keyword_to,
            $.any_identifier,
            optional($._drop_behavior)
          ),
          seq(
            $.keyword_add,
            $.keyword_value,
            optional($._if_not_exists),
            alias($._single_quote_string, $.literal),
            optional(
              seq(
                choice($.keyword_before, $.keyword_after),
                alias($._single_quote_string, $.literal)
              )
            )
          ),
          seq(
            $.keyword_rename,
            $.keyword_value,
            alias($._single_quote_string, $.literal),
            $.keyword_to,
            alias($._single_quote_string, $.literal)
          ),
          seq(
            choice(
              seq($.keyword_add, $.keyword_attribute, $.any_identifier, $.type),
              seq(
                $.keyword_drop,
                $.keyword_attribute,
                optional($._if_exists),
                $.any_identifier
              ),
              seq(
                $.keyword_alter,
                $.keyword_attribute,
                $.any_identifier,
                optional(seq($.keyword_set, $.keyword_data)),
                $.keyword_type,
                $.type
              )
            ),
            optional(seq($.keyword_collate, $.any_identifier)),
            optional($._drop_behavior)
          )
        )
      ),

    _drop_behavior: ($) => choice($.keyword_cascade, $.keyword_restrict),

    _drop_statement: ($) =>
      seq(
        choice(
          $.drop_table,
          $.drop_view,
          $.drop_index,
          $.drop_type,
          $.drop_schema,
          $.drop_database,
          $.drop_role,
          $.drop_sequence,
          $.drop_extension,
          $.drop_function,
          $.drop_policy
        )
      ),

    drop_table: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_table,
        optional($._if_exists),
        $.object_reference,
        optional($._drop_behavior)
      ),

    drop_view: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_view,
        optional($._if_exists),
        $.object_reference,
        optional($._drop_behavior)
      ),

    drop_schema: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_schema,
        optional($._if_exists),
        $.any_identifier,
        optional($._drop_behavior)
      ),

    drop_database: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_database,
        optional($._if_exists),
        $.any_identifier,
        optional($.keyword_with),
        optional($.keyword_force)
      ),

    drop_role: ($) =>
      seq(
        $.keyword_drop,
        choice($.keyword_group, $.keyword_role, $.keyword_user),
        optional($._if_exists),
        $.any_identifier
      ),

    drop_type: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_type,
        optional($._if_exists),
        $.object_reference,
        optional($._drop_behavior)
      ),

    drop_sequence: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_sequence,
        optional($._if_exists),
        $.object_reference,
        optional($._drop_behavior)
      ),

    drop_index: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_index,
        optional($.keyword_concurrently),
        optional($._if_exists),
        field("name", $.any_identifier),
        optional($._drop_behavior),
        optional(seq($.keyword_on, $.object_reference))
      ),

    drop_extension: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_extension,
        optional($._if_exists),
        comma_list($.any_identifier, true),
        optional(choice($.keyword_cascade, $.keyword_restrict))
      ),

    drop_function: ($) =>
      seq(
        $.keyword_drop,
        $.keyword_function,
        optional($._if_exists),
        $.object_reference,
        optional($._drop_behavior)
      ),

    rename_object: ($) =>
      seq($.keyword_rename, $.keyword_to, $.object_reference),

    set_schema: ($) =>
      seq($.keyword_set, $.keyword_schema, field("schema", $.any_identifier)),

    change_ownership: ($) =>
      seq($.keyword_owner, $.keyword_to, $.any_identifier),

    object_id: ($) =>
      seq(
        $.keyword_object_id,
        wrapped_in_parenthesis(
          seq(
            alias($._literal_string, $.literal),
            optional(seq(",", alias($._literal_string, $.literal)))
          )
        )
      ),

    object_reference: ($) =>
      choice(
        seq(
          field("database", $.any_identifier),
          ".",
          field("schema", $.any_identifier),
          ".",
          field("name", $.any_identifier)
        ),
        seq(
          field("schema", $.any_identifier),
          ".",
          field("name", $.any_identifier)
        ),
        field("name", $.any_identifier)
      ),

    _copy_statement: ($) =>
      seq(
        $.keyword_copy,
        $.object_reference,
        $._column_list,
        $.keyword_from,
        choice(
          $.keyword_stdin,
          alias($._literal_string, "filename"),
          seq($.keyword_program, alias($._literal_string, "command"))
        ),
        optional($.keyword_with),
        wrapped_in_parenthesis(
          repeat1(
            choice(
              seq(
                $.keyword_format,
                choice($.keyword_csv, $.keyword_binary, $.keyword_text)
              ),
              seq($.keyword_freeze, choice($.keyword_true, $.keyword_false)),
              seq(
                $.keyword_header,
                choice($.keyword_true, $.keyword_false, $.keyword_match)
              ),
              seq(
                choice(
                  $.keyword_delimiter,
                  $.keyword_null,
                  $.keyword_default,
                  $.keyword_escape,
                  $.keyword_quote,
                  $.keyword_encoding
                ),
                alias($._literal_string, $.any_identifier)
              ),
              seq(
                choice(
                  $.keyword_force_null,
                  $.keyword_force_not_null,
                  $.keyword_force_quote
                ),
                $._column_list
              )
            )
          )
        ),
        optional($.where)
      ),

    _insert_statement: ($) => seq($.insert, optional($.returning)),

    insert: ($) =>
      seq(
        $.keyword_insert,
        $.keyword_into,
        $.object_reference,
        optional($._alias),
        optional($.insert_columns),
        optional(
          seq(
            $.keyword_overriding,
            choice($.keyword_user, $.keyword_system),
            $.keyword_value
          )
        ),
        choice(
          seq($.keyword_default, $.keyword_values),
          $.insert_values,
          $._select_statement
        ),
        optional($._on_conflict)
      ),

    insert_values: ($) =>
      comma_list(
        seq(
          $.keyword_values,
          paren_list(choice($._expression, $.keyword_default), true)
        ),
        true
      ),

    insert_columns: ($) => paren_list($.column_identifier, true),

    _on_conflict: ($) =>
      seq(
        $.keyword_on,
        $.keyword_conflict,
        // todo: conflict target
        seq(
          $.keyword_do,
          choice(
            $.keyword_nothing,
            seq($.keyword_update, $._set_values, optional($.where))
          )
        )
      ),

    assignment_list: ($) => seq($.assignment, repeat(seq(",", $.assignment))),

    _set_values: ($) => seq($.keyword_set, comma_list($.assignment, true)),

    _column_list: ($) => paren_list(alias($._column, $.column), true),
    _column: ($) =>
      choice($.column_identifier, alias($._literal_string, $.literal)),

    _update_statement: ($) => seq($.update, optional($.returning)),

    _merge_statement: ($) =>
      seq(
        $.keyword_merge,
        $.keyword_into,
        $.object_reference,
        optional($._alias),
        $.keyword_using,
        choice($.subquery, $.object_reference),
        optional($._alias),
        $.keyword_on,
        optional_parenthesis(field("predicate", $._expression)),
        repeat1($.when_clause)
      ),

    when_clause: ($) =>
      seq(
        $.keyword_when,
        optional($.keyword_not),
        $.keyword_matched,
        optional(
          seq(
            $.keyword_and,
            optional_parenthesis(field("predicate", $._expression))
          )
        ),
        $.keyword_then,
        choice(
          // merge_insert
          seq(
            $.keyword_insert,
            optional(paren_list($.column_identifier, true)),
            optional(
              seq(
                $.keyword_overriding,
                choice($.keyword_system, $.keyword_user),
                $.keyword_value
              )
            ),
            choice(
              seq($.keyword_default, $.keyword_values),
              seq(
                $.keyword_values,
                paren_list(choice($._expression, $.keyword_default), true)
              )
            )
          ),
          // merge_update
          seq($.keyword_update, $._set_values),
          // merge_delete
          $.keyword_delete,

          seq($.keyword_do, $.keyword_nothing)
        )
      ),

    _optimize_statement: ($) =>
      choice($._compute_stats, $._vacuum_table, $._optimize_table),

    // Compute stats for Impala and Hive
    _compute_stats: ($) =>
      choice(
        // Hive
        seq(
          $.keyword_analyze,
          $.keyword_table,
          $.object_reference,
          optional($._partition_spec),
          $.keyword_compute,
          $.keyword_statistics,
          optional(seq($.keyword_for, $.keyword_columns)),
          optional(seq($.keyword_cache, $.keyword_metadata)),
          optional($.keyword_noscan)
        ),
        // Impala
        seq(
          $.keyword_compute,
          optional($.keyword_incremental),
          $.keyword_stats,
          $.object_reference,
          optional(
            choice(paren_list(repeat1($.field), false), $._partition_spec)
          )
        )
      ),

    _optimize_table: ($) =>
      choice(
        // Athena/Iceberg
        seq(
          $.keyword_optimize,
          $.object_reference,
          $.keyword_rewrite,
          $.keyword_data,
          $.keyword_using,
          $.keyword_bin_pack,
          optional($.where)
        ),
        // MariaDB Optimize
        seq(
          $.keyword_optimize,
          optional(
            choice(
              $.keyword_local
              //$.keyword_no_write_to_binlog,
            )
          ),
          $.keyword_table,
          $.object_reference,
          repeat(seq(",", $.object_reference))
        )
      ),

    _vacuum_table: ($) =>
      seq(
        $.keyword_vacuum,
        optional($._vacuum_option),
        $.object_reference,
        optional(paren_list($.field, false))
      ),

    _vacuum_option: ($) =>
      choice(
        seq($.keyword_full, optional(choice($.keyword_true, $.keyword_false))),
        seq(
          $.keyword_parallel,
          optional(choice($.keyword_true, $.keyword_false))
        ),
        seq(
          $.keyword_analyze,
          optional(choice($.keyword_true, $.keyword_false))
        )
        // seq($.keyword_freeze, choice($.keyword_true, $.keyword_false)),
        // seq($.keyword_skip_locked, choice($.keyword_true, $.keyword_false)),
        // seq($.keyword_truncate, choice($.keyword_true, $.keyword_false)),
        // seq($.keyword_disable_page_skipping, choice($.keyword_true, $.keyword_false)),
        // seq($.keyword_process_toast, choice($.keyword_true, $.keyword_false)),
        // seq($.keyword_index_cleanup, choice($.keyword_auto, $.keyword_on, $.keyword_off)),
      ),

    // TODO: this does not account for partitions specs like
    // (partcol1='2022-01-01', hr=11)
    // the second argument is not a $.table_option
    _partition_spec: ($) =>
      seq($.keyword_partition, paren_list($.table_option, true)),

    update: ($) =>
      seq(
        $.keyword_update,
        optional($.keyword_only),
        choice($._mysql_update_statement, $._postgres_update_statement)
      ),

    _mysql_update_statement: ($) =>
      prec(
        0,
        seq(
          comma_list($.relation, true),
          repeat($.join),
          $._set_values,
          optional($.where)
        )
      ),

    _postgres_update_statement: ($) =>
      prec(1, seq($.relation, $._set_values, optional($.from))),

    storage_location: ($) =>
      prec.right(
        seq(
          $.keyword_location,
          field("path", alias($._literal_string, $.literal)),
          optional(
            seq(
              $.keyword_cached,
              $.keyword_in,
              field("pool", alias($._literal_string, $.literal)),
              optional(
                choice(
                  $.keyword_uncached,
                  seq(
                    $.keyword_with,
                    $.keyword_replication,
                    "=",
                    field("value", alias($._natural_number, $.literal))
                  )
                )
              )
            )
          )
        )
      ),

    row_format: ($) =>
      seq(
        $.keyword_row,
        $.keyword_format,
        $.keyword_delimited,
        optional(
          seq(
            $.keyword_fields,
            $.keyword_terminated,
            $.keyword_by,
            field(
              "fields_terminated_char",
              alias($._literal_string, $.literal)
            ),
            optional(
              seq(
                $.keyword_escaped,
                $.keyword_by,
                field("escaped_char", alias($._literal_string, $.literal))
              )
            )
          )
        ),
        optional(
          seq(
            $.keyword_lines,
            $.keyword_terminated,
            $.keyword_by,
            field("row_terminated_char", alias($._literal_string, $.literal))
          )
        )
      ),

    table_partition: ($) =>
      seq(
        choice(
          // Postgres/MySQL style
          seq(
            $.keyword_partition,
            $.keyword_by,
            choice($.keyword_range, $.keyword_hash)
          ),
          // Hive style
          seq($.keyword_partitioned, $.keyword_by),
          // Spark SQL
          $.keyword_partition
        ),
        choice(
          paren_list($.any_identifier, false), // postgres & Impala (CTAS)
          $.column_definitions, // impala/hive external tables
          paren_list($._key_value_pair, true) // Spark SQL
        )
      ),

    _key_value_pair: ($) =>
      seq(
        field("key", $.any_identifier),
        "=",
        field("value", alias($._literal_string, $.literal))
      ),

    stored_as: ($) =>
      seq(
        $.keyword_stored,
        $.keyword_as,
        choice(
          $.keyword_parquet,
          $.keyword_csv,
          $.keyword_sequencefile,
          $.keyword_textfile,
          $.keyword_rcfile,
          $.keyword_orc,
          $.keyword_avro,
          $.keyword_jsonfile
        )
      ),

    assignment: ($) =>
      seq(
        field("left", alias($._qualified_field, $.field)),
        "=",
        field("right", $._expression)
      ),

    table_option: ($) =>
      choice(
        seq(
          $.keyword_default,
          $.keyword_character,
          $.keyword_set,
          $.any_identifier
        ),
        seq($.keyword_collate, $.any_identifier),
        field("name", $.keyword_default),
        seq(
          field(
            "name",
            choice($.keyword_engine, $.any_identifier, $._literal_string)
          ),
          "=",
          field("value", choice($.any_identifier, $._literal_string))
        )
      ),

    column_definitions: ($) =>
      seq(
        "(",
        comma_list($.column_definition, true),
        optional($.constraints),
        ")"
      ),

    column_definition: ($) =>
      seq(
        $.any_identifier,
        field("type", $.type),
        repeat($._column_constraint)
      ),

    _column_comment: ($) =>
      seq($.keyword_comment, alias($._literal_string, $.literal)),

    _column_constraint: ($) =>
      prec.left(
        choice(
          choice($.keyword_null, $._not_null),
          seq(
            $.keyword_references,
            $.object_reference,
            paren_list($.column_identifier, true),
            repeat(
              seq(
                $.keyword_on,
                choice($.keyword_delete, $.keyword_update),
                choice(
                  seq($.keyword_no, $.keyword_action),
                  $.keyword_restrict,
                  $.keyword_cascade,
                  seq(
                    $.keyword_set,
                    choice($.keyword_null, $.keyword_default),
                    optional(paren_list($.any_identifier, true))
                  )
                )
              )
            )
          ),
          $._default_expression,
          $._primary_key,
          $.keyword_auto_increment,
          $.direction,
          $._column_comment,
          $._check_constraint,
          seq(
            optional(seq($.keyword_generated, $.keyword_always)),
            $.keyword_as,
            $._expression
          ),
          choice($.keyword_stored, $.keyword_virtual),
          $.keyword_unique
        )
      ),

    _check_constraint: ($) =>
      seq(
        optional(seq($.keyword_constraint, $.literal)),
        $.keyword_check,
        wrapped_in_parenthesis($.binary_expression)
      ),

    _default_expression: ($) =>
      seq($.keyword_default, optional_parenthesis($._inner_default_expression)),
    _inner_default_expression: ($) =>
      choice(
        $.literal,
        $.list,
        $.cast,
        $.binary_expression,
        $.unary_expression,
        $.array,
        $.invocation,
        $.keyword_current_timestamp,
        alias($.implicit_cast, $.cast)
      ),

    constraints: ($) => seq(",", $.constraint, repeat(seq(",", $.constraint))),

    constraint: ($) =>
      choice(
        $._constraint_literal,
        $._key_constraint,
        $._primary_key_constraint,
        $._check_constraint
      ),

    _constraint_literal: ($) =>
      seq(
        $.keyword_constraint,
        field("name", $.any_identifier),
        choice(seq($._primary_key, $.ordered_columns), seq($._check_constraint))
      ),

    _primary_key_constraint: ($) => seq($._primary_key, $.ordered_columns),

    _key_constraint: ($) =>
      seq(
        choice(
          seq(
            $.keyword_unique,
            optional(
              choice(
                $.keyword_index,
                $.keyword_key,
                seq(
                  $.keyword_nulls,
                  optional($.keyword_not),
                  $.keyword_distinct
                )
              )
            )
          ),
          seq(
            optional($.keyword_foreign),
            $.keyword_key,
            optional($._if_not_exists)
          ),
          $.keyword_index
        ),
        optional(field("name", $.any_identifier)),
        $.ordered_columns,
        optional(
          seq(
            $.keyword_references,
            $.object_reference,
            paren_list($.column_identifier, true),
            repeat(
              seq(
                $.keyword_on,
                choice($.keyword_delete, $.keyword_update),
                choice(
                  seq($.keyword_no, $.keyword_action),
                  $.keyword_restrict,
                  $.keyword_cascade,
                  seq(
                    $.keyword_set,
                    choice($.keyword_null, $.keyword_default),
                    optional(paren_list($.any_identifier, true))
                  )
                )
              )
            )
          )
        )
      ),

    ordered_columns: ($) => paren_list(alias($.ordered_column, $.column), true),

    ordered_column: ($) => seq(field("name", $._column), optional($.direction)),

    all_fields: ($) => seq(optional(seq($.object_reference, ".")), "*"),

    parameter: ($) => /\?|(\$[0-9]+)/,

    case: ($) =>
      seq(
        $.keyword_case,
        choice(
          // simplified CASE x WHEN
          seq(
            $._expression,
            $.keyword_when,
            $._expression,
            $.keyword_then,
            $._expression,
            repeat(
              seq($.keyword_when, $._expression, $.keyword_then, $._expression)
            )
          ),
          // standard CASE WHEN x, where x must be a predicate
          seq(
            $.keyword_when,
            $._expression,
            $.keyword_then,
            $._expression,
            repeat(
              seq($.keyword_when, $._expression, $.keyword_then, $._expression)
            )
          )
        ),
        optional(seq($.keyword_else, $._expression)),
        $.keyword_end
      ),

    field: ($) => field("name", $.column_identifier),

    _qualified_field: ($) =>
      seq(optional($.field_qualifier), $.column_identifier),

    field_qualifier: ($) =>
      seq(prec.left(optional_parenthesis($.object_reference)), "."),

    implicit_cast: ($) => seq($._expression, "::", $.type),

    // Postgres syntax for intervals
    interval: ($) => seq($.keyword_interval, $._literal_string),

    cast: ($) =>
      seq(
        field("name", $.keyword_cast),
        wrapped_in_parenthesis(
          seq(field("parameter", $._expression), $.keyword_as, $.type)
        )
      ),

    filter_expression: ($) =>
      seq($.keyword_filter, wrapped_in_parenthesis($.where)),

    invocation: ($) =>
      prec(
        1,
        seq(
          $.object_reference,
          choice(
            // default invocation
            paren_list(
              seq(
                optional($.keyword_distinct),
                field("parameter", $.term),
                optional($.order_by)
              ),
              false
            ),
            // _aggregate_function, e.g. group_concat
            wrapped_in_parenthesis(
              seq(
                optional($.keyword_distinct),
                field("parameter", $.term),
                optional($.order_by),
                optional(
                  seq(
                    choice($.keyword_separator, ","),
                    alias($._literal_string, $.literal)
                  )
                ),
                optional($.limit)
              )
            )
          ),
          optional($.filter_expression)
        )
      ),

    exists: ($) => seq($.keyword_exists, $.subquery),

    partition_by: ($) =>
      seq($.keyword_partition, $.keyword_by, comma_list($._expression, true)),

    frame_definition: ($) =>
      seq(
        choice(
          seq($.keyword_unbounded, $.keyword_preceding),
          seq(
            field(
              "start",
              choice(
                $.any_identifier,
                $.binary_expression,
                alias($._literal_string, $.literal),
                alias($._integer, $.literal)
              )
            ),
            $.keyword_preceding
          ),
          $._current_row,
          seq(
            field(
              "end",
              choice(
                $.any_identifier,
                $.binary_expression,
                alias($._literal_string, $.literal),
                alias($._integer, $.literal)
              )
            ),
            $.keyword_following
          ),
          seq($.keyword_unbounded, $.keyword_following)
        )
      ),

    window_frame: ($) =>
      seq(
        choice($.keyword_range, $.keyword_rows, $.keyword_groups),

        choice(
          seq(
            $.keyword_between,
            $.frame_definition,
            optional(seq($.keyword_and, $.frame_definition))
          ),
          seq($.frame_definition)
        ),
        optional(
          choice(
            $._exclude_current_row,
            $._exclude_group,
            $._exclude_ties,
            $._exclude_no_others
          )
        )
      ),

    window_clause: ($) =>
      seq(
        $.keyword_window,
        $.any_identifier,
        $.keyword_as,
        $.window_specification
      ),

    window_specification: ($) =>
      wrapped_in_parenthesis(
        seq(
          optional($.partition_by),
          optional($.order_by),
          optional($.window_frame)
        )
      ),

    window_function: ($) =>
      seq(
        $.invocation,
        $.keyword_over,
        choice($.any_identifier, $.window_specification)
      ),

    _alias: ($) =>
      seq(optional($.keyword_as), field("alias", $.any_identifier)),

    from: ($) =>
      seq(
        $.keyword_from,
        optional($.keyword_only),
        comma_list($.relation, true),
        optional($.index_hint),
        repeat(
          choice($.join, $.cross_join, $.lateral_join, $.lateral_cross_join)
        ),
        optional($.where),
        optional($.group_by),
        optional($.window_clause),
        optional($.order_by),
        optional($.limit)
      ),

    relation: ($) =>
      prec.right(
        seq(
          choice(
            $.subquery,
            $.invocation,
            $.object_reference,
            wrapped_in_parenthesis($.values)
          ),
          optional(seq($._alias, optional(alias($._column_list, $.list))))
        )
      ),

    values: ($) =>
      seq($.keyword_values, $.list, optional(repeat(seq(",", $.list)))),

    index_hint: ($) =>
      seq(
        choice($.keyword_force, $.keyword_use, $.keyword_ignore),
        $.keyword_index,
        optional(seq($.keyword_for, $.keyword_join)),
        wrapped_in_parenthesis(field("index_name", $.any_identifier))
      ),

    join: ($) =>
      seq(
        optional($.keyword_natural),
        optional(
          choice(
            $.keyword_left,
            seq($.keyword_full, $.keyword_outer),
            seq($.keyword_left, $.keyword_outer),
            $.keyword_right,
            seq($.keyword_right, $.keyword_outer),
            $.keyword_inner,
            $.keyword_full
          )
        ),
        $.keyword_join,
        $.relation,
        optional($.index_hint),
        optional($.join),
        choice(
          seq($.keyword_on, field("predicate", $._expression)),
          seq($.keyword_using, alias($._column_list, $.list))
        )
      ),

    cross_join: ($) =>
      prec.right(
        seq(
          $.keyword_cross,
          $.keyword_join,
          $.relation,
          optional(
            seq(
              $.keyword_with,
              $.keyword_ordinality,
              optional(
                seq(
                  $.keyword_as,
                  field("alias", $.any_identifier),
                  paren_list($.any_identifier, false)
                )
              )
            )
          )
        )
      ),

    lateral_join: ($) =>
      seq(
        optional(
          choice(
            // lateral joins cannot be right!
            $.keyword_left,
            seq($.keyword_left, $.keyword_outer),
            $.keyword_inner
          )
        ),
        $.keyword_join,
        $.keyword_lateral,
        choice($.invocation, $.subquery),
        optional(
          choice(
            seq($.keyword_as, field("alias", $.any_identifier)),
            field("alias", $.any_identifier)
          )
        ),
        $.keyword_on,
        choice($._expression, $.keyword_true, $.keyword_false)
      ),

    lateral_cross_join: ($) =>
      seq(
        $.keyword_cross,
        $.keyword_join,
        $.keyword_lateral,
        choice($.invocation, $.subquery),
        optional(
          choice(
            seq($.keyword_as, field("alias", $.any_identifier)),
            field("alias", $.any_identifier)
          )
        )
      ),

    where: ($) => seq($.keyword_where, field("predicate", $._expression)),

    group_by: ($) =>
      seq(
        $.keyword_group,
        $.keyword_by,
        comma_list($._expression, true),
        optional($._having)
      ),

    _having: ($) => seq($.keyword_having, $._expression),

    order_by: ($) =>
      prec.right(
        seq($.keyword_order, $.keyword_by, comma_list($.order_target, true))
      ),

    order_target: ($) =>
      seq(
        $._expression,
        optional(
          seq(
            choice(
              $.direction,
              seq($.keyword_using, choice("<", ">", "<=", ">="))
            ),
            optional(
              seq($.keyword_nulls, choice($.keyword_first, $.keyword_last))
            )
          )
        )
      ),

    limit: ($) => seq($.keyword_limit, $.literal, optional($.offset)),

    offset: ($) => seq($.keyword_offset, $.literal),

    returning: ($) => seq($.keyword_returning, $.select_expression),

    grant_statement: ($) =>
      seq(
        $.keyword_grant,
        $._grantable_target_on,
        $.keyword_to,
        comma_list($.role_specification, true),
        optional(seq($.keyword_with, $.keyword_grant, $.keyword_option)),
        optional(seq($.keyword_granted, $.keyword_by, $.role_specification))
      ),

    // todo: add support for various other revoke statements
    revoke_statement: ($) =>
      seq(
        $.keyword_revoke,
        optional(
          choice(
            seq($.keyword_grant, $.keyword_option, $.keyword_for),
            seq(
              optional(
                choice($.keyword_admin, $.keyword_inherit, $.keyword_set)
              ),
              $.keyword_option,
              $.keyword_for
            )
          )
        ),
        $._grantable_target_on,
        $.keyword_from,
        comma_list($.role_specification, true),
        optional(seq($.keyword_granted, $.keyword_by, $.role_specification)),
        optional(choice($.keyword_cascade, $.keyword_restrict))
      ),

    _grantable_target_on: ($) =>
      choice(
        seq(
          $.grantable_targets,
          choice(
            $.grantable_on_table,
            $.grantable_on_function,
            $.grantable_on_all
          )
        ),
        $.any_identifier
      ),

    grantable_targets: ($) =>
      choice(
        seq($._grantable, comma_list($.any_identifier, false)),
        comma_list($.any_identifier, true)
      ),

    _grantable: ($) =>
      choice(
        comma_list(
          choice(
            $.keyword_select,
            $.keyword_insert,
            $.keyword_update,
            $.keyword_delete,
            $.keyword_truncate,
            $.keyword_references,
            $.keyword_trigger,
            $.keyword_maintain,
            $.keyword_execute,
            $.keyword_references
          ),
          true
        ),
        seq($.keyword_all, optional($.keyword_privileges))
      ),

    grantable_on_function: ($) =>
      seq(
        $.keyword_on,
        optional(
          choice($.keyword_function, $.keyword_procedure, $.keyword_routine)
        ),
        comma_list(
          seq($.object_reference, optional($.function_arguments)),
          true
        )
      ),

    grantable_on_table: ($) =>
      prec(
        1,
        seq(
          $.keyword_on,
          optional($.keyword_table),
          comma_list($.object_reference, true)
        )
      ),

    grantable_on_all: ($) =>
      seq(
        $.keyword_on,
        $.keyword_all,
        choice(
          $.keyword_tables,
          $.keyword_functions,
          $.keyword_procedures,
          $.keyword_routines
        ),
        $.keyword_in,
        $.keyword_schema,
        comma_list($.any_identifier, true)
      ),

    role_specification: ($) =>
      choice(
        seq(optional($.keyword_group), $.any_identifier),
        $.keyword_public,
        $.keyword_current_role,
        $.keyword_current_user,
        $.keyword_session_user
      ),

    _expression: ($) =>
      prec(
        1,
        choice(
          $.literal,
          alias($._qualified_field, $.field),
          $.parameter,
          $.list,
          $.case,
          $.window_function,
          $.subquery,
          $.cast,
          alias($.implicit_cast, $.cast),
          $.exists,
          $.invocation,
          $.binary_expression,
          $.subscript,
          $.unary_expression,
          $.array,
          $.interval,
          $.between_expression,
          $.parenthesized_expression,
          $.object_id
        )
      ),

    parenthesized_expression: ($) =>
      prec(2, wrapped_in_parenthesis($._expression)),

    subscript: ($) =>
      prec.left(
        "binary_is",
        seq(
          field("expression", $._expression),
          "[",
          choice(
            field("subscript", $._expression),
            seq(
              field("lower", $._expression),
              ":",
              field("upper", $._expression)
            )
          ),
          "]"
        )
      ),

    op_other: ($) =>
      token(
        choice(
          "->",
          "->>",
          "#>",
          "#>>",
          "~",
          "!~",
          "~*",
          "!~*",
          "|",
          "&",
          "#",
          "<<",
          ">>",
          "<<=",
          ">>=",
          "##",
          "<->",
          "@>",
          "<@",
          "&<",
          "&>",
          "|>>",
          "<<|",
          "&<|",
          "|&>",
          "<^",
          "^>",
          "?#",
          "?-",
          "?|",
          "?-|",
          "?||",
          "@@",
          "@@@",
          "@?",
          "#-",
          "?&",
          "?",
          "-|-",
          "||",
          "^@"
        )
      ),

    binary_expression: ($) => {
      /** @type {Array<[Rule | string, string]>} */
      const opChoices = [
        ["+", "binary_plus"],
        ["-", "binary_plus"],
        ["*", "binary_times"],
        ["/", "binary_times"],
        ["%", "binary_times"],
        ["^", "binary_exp"],
        ["=", "binary_relation"],
        ["<", "binary_relation"],
        ["<=", "binary_relation"],
        ["!=", "binary_relation"],
        [">=", "binary_relation"],
        [">", "binary_relation"],
        ["<>", "binary_relation"],
        [$.op_other, "binary_other"],
        [$.keyword_is, "binary_is"],
        [$.is_not, "binary_is"],
        [$.keyword_like, "pattern_matching"],
        [$.not_like, "pattern_matching"],
        [$.similar_to, "pattern_matching"],
        [$.not_similar_to, "pattern_matching"],
        // binary_is precedence disambiguates `(is not distinct from)` from an
        // `is (not distinct from)` with a unary `not`
        [$.distinct_from, "binary_is"],
        [$.not_distinct_from, "binary_is"],
      ];

      /** @type {Array<[Rule, string]>} */
      const clauseChoices = [
        [$.keyword_and, "clause_connective"],
        [$.keyword_or, "clause_disjunctive"],
      ];

      /** @type {Array<[Rule, string]>} */
      const binaryChoices = [
        [$.keyword_in, "binary_in"],
        [$.not_in, "binary_in"],
      ];
      return choice(
        ...opChoices.map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field("binary_expr_left", $._expression),
              field("binary_expr_operator", operator),
              field("binary_expr_right", $._expression)
            )
          )
        ),
        ...clauseChoices.map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field("binary_expr_left", $._expression),
              field("binary_expr_operator", operator),
              field("binary_expr_right", $._expression)
            )
          )
        ),
        ...binaryChoices.map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field("binary_expr_left", $._expression),
              field("binary_expr_operator", operator),
              field("binary_expr_right", choice($.list, $.subquery))
            )
          )
        )
      );
    },

    op_unary_other: ($) =>
      token(choice("|/", "||/", "@", "~", "@-@", "@@", "#", "?-", "?|", "!!")),

    unary_expression: ($) => {
      /** @type {Array<[Rule, string]>} */
      const choices = [
        [$.keyword_not, "unary_not"],
        [$.bang, "unary_not"],
        [$.keyword_any, "unary_not"],
        [$.keyword_some, "unary_not"],
        [$.keyword_all, "unary_not"],
        [$.op_unary_other, "unary_other"],
      ];

      return choice(
        ...choices.map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(field("operator", operator), field("operand", $._expression))
          )
        )
      );
    },

    between_expression: ($) => {
      /** @type {Array<[Rule, string]>} */
      const choices = [
        [$.keyword_between, "between"],
        [seq($.keyword_not, $.keyword_between), "between"],
      ];

      return choice(
        ...choices.map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field("left", $._expression),
              field("operator", operator),
              field("low", $._expression),
              $.keyword_and,
              field("high", $._expression)
            )
          )
        )
      );
    },

    not_in: ($) => seq($.keyword_not, $.keyword_in),

    subquery: ($) => wrapped_in_parenthesis($._dml_read),

    list: ($) => paren_list($._expression, false),

    literal: ($) =>
      prec(
        2,
        choice(
          $._integer,
          $._decimal_number,
          $._literal_string,
          $._bit_string,
          $._string_casting,
          $.keyword_true,
          $.keyword_false,
          $.keyword_null
        )
      ),
    _double_quote_string: (_) => /"[^"]*"/,
    // The norm specify that between two consecutive string must be a return,
    // but this is good enough.
    _single_quote_string: (_) =>
      seq(/([uU]&|[nN])?'([^']|'')*'/, repeat(/'([^']|'')*'/)),

    _postgres_escape_string: (_) => /(e|E)'([^']|\\')*'/,

    _literal_string: ($) =>
      prec(
        1,
        choice(
          $._single_quote_string,
          $._double_quote_string,
          $._dollar_quoted_string,
          $._postgres_escape_string
        )
      ),
    _natural_number: (_) => /\d+/,
    _integer: ($) =>
      seq(
        optional(choice("-", "+")),
        /(0[xX][0-9A-Fa-f]+(_[0-9A-Fa-f]+)*)|(0[oO][0-7]+(_[0-7]+)*)|(0[bB][01]+(_[01]+)*)|(\d+(_\d+)*(e[+-]?\d+(_\d+)*)?)/
      ),
    _decimal_number: ($) =>
      seq(
        optional(choice("-", "+")),
        /((\d+(_\d+)*)?[.]\d+(_\d+)*(e[+-]?\d+(_\d+)*)?)|(\d+(_\d+)*[.](e[+-]?\d+(_\d+)*)?)/
      ),
    _bit_string: ($) => seq(/[bBxX]'([^']|'')*'/, repeat(/'([^']|'')*'/)),
    // The identifier should be followed by a string (no parenthesis allowed)
    _string_casting: ($) => seq($.any_identifier, $._single_quote_string),

    bang: (_) => "!",

    any_identifier: ($) => $._any_identifier,
    column_identifier: ($) => $._any_identifier,

    _any_identifier: ($) =>
      choice(
        $._identifier,
        $._double_quote_string,
        $._sql_parameter,
        seq("`", $._identifier, "`")
      ),
    _sql_parameter: (_) => /[:$@?][a-zA-Z_][0-9a-zA-Z_]*/,
    _identifier: (_) => /[a-zA-Z_][0-9a-zA-Z_]*/,
  },
});

/**
 * @typedef {Record<string, Rule>} RuleRecord
 */

/**
 * @param {RuleRecord} $
 * @param {Rule} rule
 * @returns {ChoiceRule}
 */
function unsigned_type($, rule) {
  return choice(
    seq($.keyword_unsigned, rule),
    seq(rule, optional($.keyword_unsigned), optional($.keyword_zerofill))
  );
}

/**
 * @param {Rule} rule
 * @returns {PrecRightRule}
 */
function optional_parenthesis(rule) {
  return prec.right(choice(rule, wrapped_in_parenthesis(rule)));
}

/**
 * @param {Rule} rule
 * @returns {SeqRule}
 */
function wrapped_in_parenthesis(rule) {
  if (rule) {
    return seq("(", rule, ")");
  }
  return seq("(", ")");
}

/**
 * @param {RuleRecord} $
 * @param {Rule} rule
 * @param {string[]} params
 * @returns {PrecRightRule}
 */
function parametric_type($, rule, params = ["size"]) {
  const first = params.shift();

  if (!first) {
    throw new Error("First param should be guaranteed.");
  }

  // trust me bro
  /** @type {any} */
  const lit = $.literal;

  return prec.right(
    1,
    choice(
      rule,
      seq(
        rule,
        wrapped_in_parenthesis(
          seq(
            field(first, alias($._natural_number, lit)),
            // fill in the ", next" until done
            ...params.map((p) =>
              seq(",", field(p, alias($._natural_number, lit)))
            )
          )
        )
      )
    )
  );
}

/**
 * @param {Rule} rule
 * @param {boolean} requireFirst
 * @returns {SeqRule | ChoiceRule}
 */
function comma_list(rule, requireFirst) {
  let sequence = seq(rule, repeat(seq(",", rule)));

  if (requireFirst) {
    return sequence;
  }

  return optional(sequence);
}

/**
 * @param {Rule} rule
 * @param {boolean} requireFirst
 * @returns {SeqRule}
 */
function paren_list(rule, requireFirst) {
  return wrapped_in_parenthesis(comma_list(rule, requireFirst));
}

/**
 * @param {string} word
 * @returns {RegExp}
 */
function make_keyword(word) {
  let str = "";
  for (var i = 0; i < word.length; i++) {
    str =
      str +
      "[" +
      word.charAt(i).toLowerCase() +
      word.charAt(i).toUpperCase() +
      "]";
  }
  return new RegExp(str);
}
