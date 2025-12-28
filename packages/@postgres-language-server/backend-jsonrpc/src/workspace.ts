// Generated file, do not edit by hand, see `xtask/codegen`
import type { Transport } from "./transport";
export interface IsPathIgnoredParams {
	pgls_path: PgLSPath;
}
export interface PgLSPath {
	/**
	 * Determines the kind of the file inside Postgres Language Server. Some files are considered as configuration files, others as manifest files, and others as files to handle
	 */
	kind: FileKind;
	path: string;
	/**
	 * Whether this path (usually a file) was fixed as a result of a format/lint/check command with the `--write` filag.
	 */
	was_written: boolean;
}
export type FileKind = FileKind2[];
/**
 * The priority of the file
 */
export type FileKind2 = "Config" | "Ignore" | "Inspectable" | "Handleable";
export interface RegisterProjectFolderParams {
	path?: string;
	setAsCurrentWorkspace: boolean;
}
export type ProjectKey = string;
export interface GetFileContentParams {
	path: PgLSPath;
}
export interface PullFileDiagnosticsParams {
	categories: RuleCategories;
	max_diagnostics: number;
	only: RuleCode[];
	path: PgLSPath;
	skip: RuleCode[];
}
export type RuleCategories = RuleCategory[];
export type RuleCode = string;
export type RuleCategory = "Lint" | "Action" | "Transformation";
export interface PullDiagnosticsResult {
	diagnostics: Diagnostic[];
	skipped_diagnostics: number;
}
/**
 * Serializable representation for a [Diagnostic](super::Diagnostic).
 */
export interface Diagnostic {
	advices: Advices;
	category?: Category;
	description: string;
	location: Location;
	message: MarkupBuf;
	severity: Severity;
	source?: Diagnostic;
	tags: DiagnosticTags;
	verboseAdvices: Advices;
}
/**
 * Implementation of [Visitor] collecting serializable [Advice] into a vector.
 */
export interface Advices {
	advices: Advice[];
}
export type Category =
	| "lint/safety/addSerialColumn"
	| "lint/safety/addingFieldWithDefault"
	| "lint/safety/addingForeignKeyConstraint"
	| "lint/safety/addingNotNullField"
	| "lint/safety/addingPrimaryKeyConstraint"
	| "lint/safety/addingRequiredField"
	| "lint/safety/banCharField"
	| "lint/safety/banConcurrentIndexCreationInTransaction"
	| "lint/safety/banDropColumn"
	| "lint/safety/banDropDatabase"
	| "lint/safety/banDropNotNull"
	| "lint/safety/banDropTable"
	| "lint/safety/banTruncateCascade"
	| "lint/safety/changingColumnType"
	| "lint/safety/constraintMissingNotValid"
	| "lint/safety/creatingEnum"
	| "lint/safety/disallowUniqueConstraint"
	| "lint/safety/lockTimeoutWarning"
	| "lint/safety/multipleAlterTable"
	| "lint/safety/preferBigInt"
	| "lint/safety/preferBigintOverInt"
	| "lint/safety/preferBigintOverSmallint"
	| "lint/safety/preferIdentity"
	| "lint/safety/preferJsonb"
	| "lint/safety/preferRobustStmts"
	| "lint/safety/preferTextField"
	| "lint/safety/preferTimestamptz"
	| "lint/safety/renamingColumn"
	| "lint/safety/renamingTable"
	| "lint/safety/requireConcurrentIndexCreation"
	| "lint/safety/requireConcurrentIndexDeletion"
	| "lint/safety/runningStatementWhileHoldingAccessExclusive"
	| "lint/safety/transactionNesting"
	| "splinter/performance/authRlsInitplan"
	| "splinter/performance/duplicateIndex"
	| "splinter/performance/multiplePermissivePolicies"
	| "splinter/performance/noPrimaryKey"
	| "splinter/performance/tableBloat"
	| "splinter/performance/unindexedForeignKeys"
	| "splinter/performance/unusedIndex"
	| "splinter/security/authUsersExposed"
	| "splinter/security/extensionInPublic"
	| "splinter/security/extensionVersionsOutdated"
	| "splinter/security/fkeyToAuthUnique"
	| "splinter/security/foreignTableInApi"
	| "splinter/security/functionSearchPathMutable"
	| "splinter/security/insecureQueueExposedInApi"
	| "splinter/security/materializedViewInApi"
	| "splinter/security/policyExistsRlsDisabled"
	| "splinter/security/rlsDisabledInPublic"
	| "splinter/security/rlsEnabledNoPolicy"
	| "splinter/security/rlsReferencesUserMetadata"
	| "splinter/security/securityDefinerView"
	| "splinter/security/unsupportedRegTypes"
	| "stdin"
	| "check"
	| "configuration"
	| "database/connection"
	| "internalError/io"
	| "internalError/runtime"
	| "internalError/fs"
	| "flags/invalid"
	| "project"
	| "typecheck"
	| "plpgsql_check"
	| "internalError/panic"
	| "syntax"
	| "dummy"
	| "lint"
	| "lint/performance"
	| "lint/safety"
	| "splinter"
	| "splinter/performance"
	| "splinter/security";
export interface Location {
	path?: Resource_for_String;
	sourceCode?: string;
	span?: TextRange;
}
export type MarkupBuf = MarkupNodeBuf[];
/**
 * The severity to associate to a diagnostic.
 */
export type Severity = "hint" | "information" | "warning" | "error" | "fatal";
export type DiagnosticTags = DiagnosticTag[];
/**
	* Serializable representation of a [Diagnostic](super::Diagnostic) advice

See the [Visitor] trait for additional documentation on all the supported advice types. 
	 */
export type Advice =
	| { log: [LogCategory, MarkupBuf] }
	| { list: MarkupBuf[] }
	| { frame: Location }
	| { diff: TextEdit }
	| { backtrace: [MarkupBuf, Backtrace] }
	| { command: string }
	| { group: [MarkupBuf, Advices] };
/**
 * Represents the resource a diagnostic is associated with.
 */
export type Resource_for_String =
	| "database"
	| "argv"
	| "memory"
	| { file: string };
export type TextRange = [TextSize, TextSize];
export interface MarkupNodeBuf {
	content: string;
	elements: MarkupElement[];
}
/**
 * Internal enum used to automatically generate bit offsets for [DiagnosticTags] and help with the implementation of `serde` and `schemars` for tags.
 */
export type DiagnosticTag =
	| "fixable"
	| "internal"
	| "unnecessaryCode"
	| "deprecatedCode"
	| "verbose";
/**
 * The category for a log advice, defines how the message should be presented to the user.
 */
export type LogCategory = "none" | "info" | "warn" | "error";
export interface TextEdit {
	dictionary: string;
	ops: CompressedOp[];
}
export type Backtrace = BacktraceFrame[];
export type TextSize = number;
/**
 * Enumeration of all the supported markup elements
 */
export type MarkupElement =
	| "Emphasis"
	| "Dim"
	| "Italic"
	| "Underline"
	| "Error"
	| "Success"
	| "Warn"
	| "Info"
	| "Debug"
	| "Trace"
	| "Inverse"
	| { Hyperlink: { href: string } };
export type CompressedOp =
	| { diffOp: DiffOp }
	| { equalLines: { line_count: number } };
/**
 * Serializable representation of a backtrace frame.
 */
export interface BacktraceFrame {
	ip: number;
	symbols: BacktraceSymbol[];
}
export type DiffOp =
	| { equal: { range: TextRange } }
	| { insert: { range: TextRange } }
	| { delete: { range: TextRange } };
/**
 * Serializable representation of a backtrace frame symbol.
 */
export interface BacktraceSymbol {
	colno?: number;
	filename?: string;
	lineno?: number;
	name?: string;
}
export interface GetCompletionsParams {
	/**
	 * The File for which a completion is requested.
	 */
	path: PgLSPath;
	/**
	 * The Cursor position in the file for which a completion is requested.
	 */
	position: TextSize;
}
export interface CompletionsResult {
	items: CompletionItem[];
}
export interface CompletionItem {
	completion_text?: CompletionText;
	description: string;
	detail?: string;
	kind: CompletionItemKind;
	label: string;
	preselected: boolean;
	/**
	 * String used for sorting by LSP clients.
	 */
	sort_text: string;
}
/**
	* The text that the editor should fill in. If `None`, the `label` should be used. Tables, for example, might have different completion_texts:

label: "users", description: "Schema: auth", completion_text: "auth.users". 
	 */
export interface CompletionText {
	is_snippet: boolean;
	/**
	 * A `range` is required because some editors replace the current token, others naively insert the text. Having a range where start == end makes it an insertion.
	 */
	range: TextRange;
	text: string;
}
export type CompletionItemKind =
	| "table"
	| "function"
	| "column"
	| "schema"
	| "policy"
	| "role";
export interface UpdateSettingsParams {
	configuration: PartialConfiguration;
	gitignore_matches: string[];
	vcs_base_path?: string;
	workspace_directory?: string;
}
/**
 * The configuration that is contained inside the configuration file.
 */
export interface PartialConfiguration {
	/**
	 * A field for the [JSON schema](https://json-schema.org/) specification
	 */
	$schema?: string;
	/**
	 * The configuration of the database connection
	 */
	db?: PartialDatabaseConfiguration;
	/**
	 * A list of paths to other JSON files, used to extends the current configuration.
	 */
	extends?: StringSet;
	/**
	 * The configuration of the filesystem
	 */
	files?: PartialFilesConfiguration;
	/**
	 * The configuration for the linter
	 */
	linter?: PartialLinterConfiguration;
	/**
	 * Configure migrations
	 */
	migrations?: PartialMigrationsConfiguration;
	/**
	 * The configuration for type checking
	 */
	plpgsqlCheck?: PartialPlPgSqlCheckConfiguration;
	/**
	 * The configuration for splinter
	 */
	splinter?: PartialSplinterConfiguration;
	/**
	 * The configuration for type checking
	 */
	typecheck?: PartialTypecheckConfiguration;
	/**
	 * The configuration of the VCS integration
	 */
	vcs?: PartialVcsConfiguration;
}
/**
 * The configuration of the database connection.
 */
export interface PartialDatabaseConfiguration {
	allowStatementExecutionsAgainst?: StringSet;
	/**
	 * The connection timeout in seconds.
	 */
	connTimeoutSecs?: number;
	/**
	 * A connection string that encodes the full connection setup. When provided, it takes precedence over the individual fields.
	 */
	connectionString?: string;
	/**
	 * The name of the database.
	 */
	database?: string;
	/**
	 * The host of the database. Required if you want database-related features. All else falls back to sensible defaults.
	 */
	host?: string;
	/**
	 * The password to connect to the database.
	 */
	password?: string;
	/**
	 * The port of the database.
	 */
	port?: number;
	/**
	 * The username to connect to the database.
	 */
	username?: string;
}
export type StringSet = string[];
/**
 * The configuration of the filesystem
 */
export interface PartialFilesConfiguration {
	/**
	 * A list of Unix shell style patterns. Will ignore files/folders that will match these patterns.
	 */
	ignore?: StringSet;
	/**
	 * A list of Unix shell style patterns. Will handle only those files/folders that will match these patterns.
	 */
	include?: StringSet;
	/**
	 * The maximum allowed size for source code files in bytes. Files above this limit will be ignored for performance reasons. Defaults to 1 MiB
	 */
	maxSize?: number;
}
export interface PartialLinterConfiguration {
	/**
	 * if `false`, it disables the feature and the linter won't be executed. `true` by default
	 */
	enabled?: boolean;
	/**
	 * A list of Unix shell style patterns. The linter will ignore files/folders that will match these patterns.
	 */
	ignore?: StringSet;
	/**
	 * A list of Unix shell style patterns. The linter will include files/folders that will match these patterns.
	 */
	include?: StringSet;
	/**
	 * List of rules
	 */
	rules?: LinterRules;
}
/**
 * The configuration of the filesystem
 */
export interface PartialMigrationsConfiguration {
	/**
	 * Ignore any migrations before this timestamp
	 */
	after?: number;
	/**
	 * The directory where the migration files are stored
	 */
	migrationsDir?: string;
}
/**
 * The configuration for type checking.
 */
export interface PartialPlPgSqlCheckConfiguration {
	/**
	 * if `false`, it disables the feature and pglpgsql_check won't be executed. `true` by default
	 */
	enabled?: boolean;
}
export interface PartialSplinterConfiguration {
	/**
	 * if `false`, it disables the feature and the linter won't be executed. `true` by default
	 */
	enabled?: boolean;
	/**
	 * List of rules
	 */
	rules?: SplinterRules;
}
/**
 * The configuration for type checking.
 */
export interface PartialTypecheckConfiguration {
	/**
	 * if `false`, it disables the feature and the typechecker won't be executed. `true` by default
	 */
	enabled?: boolean;
	/**
	 * Default search path schemas for type checking. Can be a list of schema names or glob patterns like ["public", "app_*"]. If not specified, defaults to ["public"].
	 */
	searchPath?: StringSet;
}
/**
 * Set of properties to integrate with a VCS software.
 */
export interface PartialVcsConfiguration {
	/**
	 * The kind of client.
	 */
	clientKind?: VcsClientKind;
	/**
	 * The main branch of the project
	 */
	defaultBranch?: string;
	/**
	 * Whether we should integrate itself with the VCS client
	 */
	enabled?: boolean;
	/**
	* The folder where we should check for VCS files. By default, we will use the same folder where `postgres-language-server.jsonc` was found.

If we can't find the configuration, it will attempt to use the current working directory. If no current working directory can't be found, we won't use the VCS integration, and a diagnostic will be emitted 
	 */
	root?: string;
	/**
	 * Whether we should use the VCS ignore file. When [true], we will ignore the files specified in the ignore file.
	 */
	useIgnoreFile?: boolean;
}
export interface LinterRules {
	/**
	 * It enables ALL rules. The rules that belong to `nursery` won't be enabled.
	 */
	all?: boolean;
	/**
	 * It enables the lint rules recommended by Postgres Language Server. `true` by default.
	 */
	recommended?: boolean;
	safety?: Safety;
}
export interface SplinterRules {
	/**
	 * It enables ALL rules. The rules that belong to `nursery` won't be enabled.
	 */
	all?: boolean;
	performance?: Performance;
	/**
	 * It enables the lint rules recommended by Postgres Language Server. `true` by default.
	 */
	recommended?: boolean;
	security?: Security;
}
export type VcsClientKind = "git";
/**
 * A list of rules that belong to this group
 */
export interface Safety {
	/**
	 * Adding a column with a SERIAL type or GENERATED ALWAYS AS ... STORED causes a full table rewrite.
	 */
	addSerialColumn?: RuleConfiguration_for_Null;
	/**
	 * Adding a column with a DEFAULT value may lead to a table rewrite while holding an ACCESS EXCLUSIVE lock.
	 */
	addingFieldWithDefault?: RuleConfiguration_for_Null;
	/**
	 * Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes.
	 */
	addingForeignKeyConstraint?: RuleConfiguration_for_Null;
	/**
	 * Setting a column NOT NULL blocks reads while the table is scanned.
	 */
	addingNotNullField?: RuleConfiguration_for_Null;
	/**
	 * Adding a primary key constraint results in locks and table rewrites.
	 */
	addingPrimaryKeyConstraint?: RuleConfiguration_for_Null;
	/**
	 * Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required.
	 */
	addingRequiredField?: RuleConfiguration_for_Null;
	/**
	 * It enables ALL rules for this group.
	 */
	all?: boolean;
	/**
	 * Using CHAR(n) or CHARACTER(n) types is discouraged.
	 */
	banCharField?: RuleConfiguration_for_Null;
	/**
	 * Concurrent index creation is not allowed within a transaction.
	 */
	banConcurrentIndexCreationInTransaction?: RuleConfiguration_for_Null;
	/**
	 * Dropping a column may break existing clients.
	 */
	banDropColumn?: RuleConfiguration_for_Null;
	/**
	 * Dropping a database may break existing clients (and everything else, really).
	 */
	banDropDatabase?: RuleConfiguration_for_Null;
	/**
	 * Dropping a NOT NULL constraint may break existing clients.
	 */
	banDropNotNull?: RuleConfiguration_for_Null;
	/**
	 * Dropping a table may break existing clients.
	 */
	banDropTable?: RuleConfiguration_for_Null;
	/**
	 * Using TRUNCATE's CASCADE option will truncate any tables that are also foreign-keyed to the specified tables.
	 */
	banTruncateCascade?: RuleConfiguration_for_Null;
	/**
	 * Changing a column type may break existing clients.
	 */
	changingColumnType?: RuleConfiguration_for_Null;
	/**
	 * Adding constraints without NOT VALID blocks all reads and writes.
	 */
	constraintMissingNotValid?: RuleConfiguration_for_Null;
	/**
	 * Creating enum types is not recommended for new applications.
	 */
	creatingEnum?: RuleConfiguration_for_Null;
	/**
	 * Disallow adding a UNIQUE constraint without using an existing index.
	 */
	disallowUniqueConstraint?: RuleConfiguration_for_Null;
	/**
	 * Taking a dangerous lock without setting a lock timeout can cause indefinite blocking.
	 */
	lockTimeoutWarning?: RuleConfiguration_for_Null;
	/**
	 * Multiple ALTER TABLE statements on the same table should be combined into a single statement.
	 */
	multipleAlterTable?: RuleConfiguration_for_Null;
	/**
	 * Prefer BIGINT over smaller integer types.
	 */
	preferBigInt?: RuleConfiguration_for_Null;
	/**
	 * Prefer BIGINT over INT/INTEGER types.
	 */
	preferBigintOverInt?: RuleConfiguration_for_Null;
	/**
	 * Prefer BIGINT over SMALLINT types.
	 */
	preferBigintOverSmallint?: RuleConfiguration_for_Null;
	/**
	 * Prefer using IDENTITY columns over serial columns.
	 */
	preferIdentity?: RuleConfiguration_for_Null;
	/**
	 * Prefer JSONB over JSON types.
	 */
	preferJsonb?: RuleConfiguration_for_Null;
	/**
	 * Prefer statements with guards for robustness in migrations.
	 */
	preferRobustStmts?: RuleConfiguration_for_Null;
	/**
	 * Prefer using TEXT over VARCHAR(n) types.
	 */
	preferTextField?: RuleConfiguration_for_Null;
	/**
	 * Prefer TIMESTAMPTZ over TIMESTAMP types.
	 */
	preferTimestamptz?: RuleConfiguration_for_Null;
	/**
	 * It enables the recommended rules for this group
	 */
	recommended?: boolean;
	/**
	 * Renaming columns may break existing queries and application code.
	 */
	renamingColumn?: RuleConfiguration_for_Null;
	/**
	 * Renaming tables may break existing queries and application code.
	 */
	renamingTable?: RuleConfiguration_for_Null;
	/**
	 * Creating indexes non-concurrently can lock the table for writes.
	 */
	requireConcurrentIndexCreation?: RuleConfiguration_for_Null;
	/**
	 * Dropping indexes non-concurrently can lock the table for reads.
	 */
	requireConcurrentIndexDeletion?: RuleConfiguration_for_Null;
	/**
	 * Running additional statements while holding an ACCESS EXCLUSIVE lock blocks all table access.
	 */
	runningStatementWhileHoldingAccessExclusive?: RuleConfiguration_for_Null;
	/**
	 * Detects problematic transaction nesting that could lead to unexpected behavior.
	 */
	transactionNesting?: RuleConfiguration_for_Null;
}
/**
 * A list of rules that belong to this group
 */
export interface Performance {
	/**
	 * It enables ALL rules for this group.
	 */
	all?: boolean;
	/**
	 * Auth RLS Initialization Plan: Detects if calls to `current_setting()` and `auth.()` in RLS policies are being unnecessarily re-evaluated for each row
	 */
	authRlsInitplan?: RuleConfiguration_for_Null;
	/**
	 * Duplicate Index: Detects cases where two ore more identical indexes exist.
	 */
	duplicateIndex?: RuleConfiguration_for_Null;
	/**
	 * Multiple Permissive Policies: Detects if multiple permissive row level security policies are present on a table for the same `role` and `action` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query.
	 */
	multiplePermissivePolicies?: RuleConfiguration_for_Null;
	/**
	 * No Primary Key: Detects if a table does not have a primary key. Tables without a primary key can be inefficient to interact with at scale.
	 */
	noPrimaryKey?: RuleConfiguration_for_Null;
	/**
	 * It enables the recommended rules for this group
	 */
	recommended?: boolean;
	/**
	 * Table Bloat: Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster.
	 */
	tableBloat?: RuleConfiguration_for_Null;
	/**
	 * Unindexed foreign keys: Identifies foreign key constraints without a covering index, which can impact database performance.
	 */
	unindexedForeignKeys?: RuleConfiguration_for_Null;
	/**
	 * Unused Index: Detects if an index has never been used and may be a candidate for removal.
	 */
	unusedIndex?: RuleConfiguration_for_Null;
}
/**
 * A list of rules that belong to this group
 */
export interface Security {
	/**
	 * It enables ALL rules for this group.
	 */
	all?: boolean;
	/**
	 * Exposed Auth Users: Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security.
	 */
	authUsersExposed?: RuleConfiguration_for_Null;
	/**
	 * Extension in Public: Detects extensions installed in the `public` schema.
	 */
	extensionInPublic?: RuleConfiguration_for_Null;
	/**
	 * Extension Versions Outdated: Detects extensions that are not using the default (recommended) version.
	 */
	extensionVersionsOutdated?: RuleConfiguration_for_Null;
	/**
	 * Foreign Key to Auth Unique Constraint: Detects user defined foreign keys to unique constraints in the auth schema.
	 */
	fkeyToAuthUnique?: RuleConfiguration_for_Null;
	/**
	 * Foreign Table in API: Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies.
	 */
	foreignTableInApi?: RuleConfiguration_for_Null;
	/**
	 * Function Search Path Mutable: Detects functions where the search_path parameter is not set.
	 */
	functionSearchPathMutable?: RuleConfiguration_for_Null;
	/**
	 * Insecure Queue Exposed in API: Detects cases where an insecure Queue is exposed over Data APIs
	 */
	insecureQueueExposedInApi?: RuleConfiguration_for_Null;
	/**
	 * Materialized View in API: Detects materialized views that are accessible over the Data APIs.
	 */
	materializedViewInApi?: RuleConfiguration_for_Null;
	/**
	 * Policy Exists RLS Disabled: Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table.
	 */
	policyExistsRlsDisabled?: RuleConfiguration_for_Null;
	/**
	 * It enables the recommended rules for this group
	 */
	recommended?: boolean;
	/**
	 * RLS Disabled in Public: Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST
	 */
	rlsDisabledInPublic?: RuleConfiguration_for_Null;
	/**
	 * RLS Enabled No Policy: Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created.
	 */
	rlsEnabledNoPolicy?: RuleConfiguration_for_Null;
	/**
	 * RLS references user metadata: Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy.
	 */
	rlsReferencesUserMetadata?: RuleConfiguration_for_Null;
	/**
	 * Security Definer View: Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user
	 */
	securityDefinerView?: RuleConfiguration_for_Null;
	/**
	 * Unsupported reg types: Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade.
	 */
	unsupportedRegTypes?: RuleConfiguration_for_Null;
}
export type RuleConfiguration_for_Null =
	| RulePlainConfiguration
	| RuleWithOptions_for_Null;
export type RulePlainConfiguration = "warn" | "error" | "info" | "off";
export interface RuleWithOptions_for_Null {
	/**
	 * The severity of the emitted diagnostics by the rule
	 */
	level: RulePlainConfiguration;
	/**
	 * Rule's options
	 */
	options: null;
}
export interface OpenFileParams {
	content: string;
	path: PgLSPath;
	version: number;
}
export interface ChangeFileParams {
	content: string;
	path: PgLSPath;
	version: number;
}
export interface CloseFileParams {
	path: PgLSPath;
}
export type Configuration = PartialConfiguration;
export interface Workspace {
	isPathIgnored(params: IsPathIgnoredParams): Promise<boolean>;
	registerProjectFolder(
		params: RegisterProjectFolderParams,
	): Promise<ProjectKey>;
	getFileContent(params: GetFileContentParams): Promise<string>;
	pullFileDiagnostics(
		params: PullFileDiagnosticsParams,
	): Promise<PullDiagnosticsResult>;
	getCompletions(params: GetCompletionsParams): Promise<CompletionsResult>;
	updateSettings(params: UpdateSettingsParams): Promise<void>;
	openFile(params: OpenFileParams): Promise<void>;
	changeFile(params: ChangeFileParams): Promise<void>;
	closeFile(params: CloseFileParams): Promise<void>;
	destroy(): void;
}
export function createWorkspace(transport: Transport): Workspace {
	return {
		isPathIgnored(params) {
			return transport.request("pgls/is_path_ignored", params);
		},
		registerProjectFolder(params) {
			return transport.request("pgls/register_project_folder", params);
		},
		getFileContent(params) {
			return transport.request("pgls/get_file_content", params);
		},
		pullFileDiagnostics(params) {
			return transport.request("pgls/pull_file_diagnostics", params);
		},
		getCompletions(params) {
			return transport.request("pgls/get_completions", params);
		},
		updateSettings(params) {
			return transport.request("pgls/update_settings", params);
		},
		openFile(params) {
			return transport.request("pgls/open_file", params);
		},
		changeFile(params) {
			return transport.request("pgls/change_file", params);
		},
		closeFile(params) {
			return transport.request("pgls/close_file", params);
		},
		destroy() {
			transport.destroy();
		},
	};
}
