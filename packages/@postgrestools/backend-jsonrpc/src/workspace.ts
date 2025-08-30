// Generated file, do not edit by hand, see `xtask/codegen`
import type { Transport } from "./transport";
export interface IsPathIgnoredParams {
	pgt_path: PgTPath;
}
export interface PgTPath {
	/**
	 * Determines the kind of the file inside Postgres Tools. Some files are considered as configuration files, others as manifest files, and others as files to handle
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
	path: PgTPath;
}
export interface PullDiagnosticsParams {
	categories: RuleCategories;
	max_diagnostics: number;
	only: RuleCode[];
	path: PgTPath;
	skip: RuleCode[];
}
export type RuleCategories = RuleCategory[];
export type RuleCode = string;
export type RuleCategory = "Lint" | "Action" | "Transformation";
export interface PullDiagnosticsResult {
	diagnostics: Diagnostic[];
	errors: number;
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
	| "lint/safety/addingRequiredField"
	| "lint/safety/banDropColumn"
	| "lint/safety/banDropDatabase"
	| "lint/safety/banDropNotNull"
	| "lint/safety/banDropTable"
	| "lint/safety/banTruncateCascade"
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
	| "lint/safety";
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
export type Resource_for_String = "argv" | "memory" | { file: string };
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
	path: PgTPath;
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
	 * A list of Unix shell style patterns. The formatter will ignore files/folders that will match these patterns.
	 */
	ignore?: StringSet;
	/**
	 * A list of Unix shell style patterns. The formatter will include files/folders that will match these patterns.
	 */
	include?: StringSet;
	/**
	 * List of rules
	 */
	rules?: Rules;
	/**
	 * Default search path schemas for type checking. Can be a list of schema names or glob patterns like ["public", "app_*"]. If not specified, defaults to ["public"].
	 */
	searchPathPatterns?: StringSet;
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
	* The folder where we should check for VCS files. By default, we will use the same folder where `postgrestools.jsonc` was found.

If we can't find the configuration, it will attempt to use the current working directory. If no current working directory can't be found, we won't use the VCS integration, and a diagnostic will be emitted 
	 */
	root?: string;
	/**
	 * Whether we should use the VCS ignore file. When [true], we will ignore the files specified in the ignore file.
	 */
	useIgnoreFile?: boolean;
}
export interface Rules {
	/**
	 * It enables ALL rules. The rules that belong to `nursery` won't be enabled.
	 */
	all?: boolean;
	/**
	 * It enables the lint rules recommended by Postgres Tools. `true` by default.
	 */
	recommended?: boolean;
	safety?: Safety;
}
export type VcsClientKind = "git";
/**
 * A list of rules that belong to this group
 */
export interface Safety {
	/**
	 * Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required.
	 */
	addingRequiredField?: RuleConfiguration_for_Null;
	/**
	 * It enables ALL rules for this group.
	 */
	all?: boolean;
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
	 * It enables the recommended rules for this group
	 */
	recommended?: boolean;
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
	path: PgTPath;
	version: number;
}
export interface ChangeFileParams {
	content: string;
	path: PgTPath;
	version: number;
}
export interface CloseFileParams {
	path: PgTPath;
}
export type Configuration = PartialConfiguration;
export interface Workspace {
	isPathIgnored(params: IsPathIgnoredParams): Promise<boolean>;
	registerProjectFolder(
		params: RegisterProjectFolderParams,
	): Promise<ProjectKey>;
	getFileContent(params: GetFileContentParams): Promise<string>;
	pullDiagnostics(
		params: PullDiagnosticsParams,
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
			return transport.request("pgt/is_path_ignored", params);
		},
		registerProjectFolder(params) {
			return transport.request("pgt/register_project_folder", params);
		},
		getFileContent(params) {
			return transport.request("pgt/get_file_content", params);
		},
		pullDiagnostics(params) {
			return transport.request("pgt/pull_diagnostics", params);
		},
		getCompletions(params) {
			return transport.request("pgt/get_completions", params);
		},
		updateSettings(params) {
			return transport.request("pgt/update_settings", params);
		},
		openFile(params) {
			return transport.request("pgt/open_file", params);
		},
		changeFile(params) {
			return transport.request("pgt/change_file", params);
		},
		closeFile(params) {
			return transport.request("pgt/close_file", params);
		},
		destroy() {
			transport.destroy();
		},
	};
}
