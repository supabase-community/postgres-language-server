// Generated file, do not edit by hand, see `xtask/codegen`
export interface SchemaCache {
	columns?: Column[];
	extensions?: Extension[];
	functions?: Function[];
	indexes?: Index[];
	policies?: Policy[];
	roles?: Role[];
	schemas?: Schema[];
	sequences?: Sequence[];
	tables?: Table[];
	triggers?: Trigger[];
	types?: PostgresType[];
	version?: Version;
}
export interface Column {
	/**
	 * What type of class does this column belong to?
	 */
	class_kind: ColumnClassKind;
	/**
	 * Comment inserted via `COMMENT ON COLUMN my_table.my_comment '...'`, if present.
	 */
	comment?: string;
	/**
	 * The Default "value" of the column. Might be a function call, hence "_expr".
	 */
	default_expr?: string;
	is_nullable: boolean;
	is_primary_key: boolean;
	is_unique: boolean;
	name: string;
	/**
	 * the column number in the table
	 */
	number: number;
	schema_name: string;
	table_name: string;
	table_oid: number;
	type_id: number;
	type_name?: string;
	varchar_length?: number;
}
export interface Extension {
	comment?: string;
	default_version: string;
	installed_version?: string;
	name: string;
	schema?: string;
}
export interface Function {
	/**
	 * The Rust representation of the function's arguments.
	 */
	args: FunctionArgs;
	/**
	 * Comma-separated list of argument types, in the form required for a CREATE FUNCTION statement. For example, `"text, smallint"`. `None` if the function doesn't take any arguments.
	 */
	argument_types?: string;
	/**
	 * See `Behavior`.
	 */
	behavior: Behavior;
	/**
	 * The body of the function – the `declare [..] begin [..] end [..]` block.` Not set for internal functions.
	 */
	body?: string;
	/**
	 * The full definition of the function. Includes the full `CREATE OR REPLACE...` shenanigans. Not set for internal functions.
	 */
	definition?: string;
	/**
	 * The Id (`oid`).
	 */
	id: number;
	/**
	 * Comma-separated list of argument types, in the form required to identify a function in an ALTER FUNCTION statement. For example, `"text, smallint"`. `None` if the function doesn't take any arguments.
	 */
	identity_argument_types?: string;
	/**
	 * Does the function returns multiple values of a data type?
	 */
	is_set_returning_function: boolean;
	kind: ProcKind;
	/**
	 * e.g. `plpgsql/sql` or `internal`.
	 */
	language: string;
	/**
	 * The name of the function.
	 */
	name: string;
	/**
	 * The return type, for example "text", "trigger", or "void".
	 */
	return_type?: string;
	/**
	 * An ID identifying the return type. For example, `2275` refers to `cstring`. 2278 refers to `void`.
	 */
	return_type_id?: number;
	/**
	 * If the return type is a composite type, this will point the matching entry's `oid` column in the `pg_class` table. `None` if the function does not return a composite type.
	 */
	return_type_relation_id?: number;
	/**
	 * The name of the schema the function belongs to.
	 */
	schema: string;
	/**
	 * Is the function's security set to `Definer` (true) or `Invoker` (false)?
	 */
	security_definer: boolean;
}
export interface Index {
	id: number;
	name: string;
	schema: string;
	table_name: string;
}
export interface Policy {
	command: PolicyCommand;
	is_permissive: boolean;
	name: string;
	role_names: string[];
	schema_name: string;
	security_qualification?: string;
	table_name: string;
	with_check?: string;
}
export interface Role {
	can_bypass_rls: boolean;
	can_create_db: boolean;
	can_create_roles: boolean;
	can_login: boolean;
	comment?: string;
	has_member: string[];
	is_super_user: boolean;
	member_of: string[];
	name: string;
}
export interface Schema {
	allowed_creators: string[];
	allowed_users: string[];
	comment?: string;
	function_count: number;
	id: number;
	name: string;
	owner: string;
	table_count: number;
	total_size: string;
	view_count: number;
}
export interface Sequence {
	id: number;
	name: string;
	schema: string;
}
export interface Table {
	bytes: number;
	comment?: string;
	dead_rows_estimate: number;
	id: number;
	live_rows_estimate: number;
	name: string;
	replica_identity: ReplicaIdentity;
	rls_enabled: boolean;
	rls_forced: boolean;
	schema: string;
	size: string;
	table_kind: TableKind;
}
export interface Trigger {
	affected: TriggerAffected;
	events: TriggerEvent[];
	name: string;
	proc_name: string;
	proc_schema: string;
	table_name: string;
	table_schema: string;
	timing: TriggerTiming;
}
export interface PostgresType {
	attributes: TypeAttributes;
	comment?: string;
	enums: Enums;
	format: string;
	id: number;
	name: string;
	schema: string;
}
export interface Version {
	active_connections?: number;
	major_version?: number;
	max_connections?: number;
	version?: string;
	version_num?: number;
}
export type ColumnClassKind =
	| "OrdinaryTable"
	| "View"
	| "MaterializedView"
	| "ForeignTable"
	| "PartitionedTable";
export interface FunctionArgs {
	args: FunctionArg[];
}
/**
 * `Behavior` describes the characteristics of the function. Is it deterministic? Does it changed due to side effects, and if so, when?
 */
export type Behavior = "Immutable" | "Stable" | "Volatile";
export type ProcKind = "Function" | "Aggregate" | "Window" | "Procedure";
export type PolicyCommand = "Select" | "Insert" | "Update" | "Delete" | "All";
export type ReplicaIdentity = "Default" | "Index" | "Full" | "Nothing";
export type TableKind =
	| "Ordinary"
	| "View"
	| "MaterializedView"
	| "Partitioned";
export type TriggerAffected = "Row" | "Statement";
export type TriggerEvent = "Insert" | "Delete" | "Update" | "Truncate";
export type TriggerTiming = "Before" | "After" | "Instead";
export interface TypeAttributes {
	attrs: PostgresTypeAttribute[];
}
export interface Enums {
	values: string[];
}
export interface FunctionArg {
	has_default?: boolean;
	/**
	 * `in`, `out`, or `inout`.
	 */
	mode: string;
	name: string;
	/**
	 * Refers to the argument type's ID in the `pg_type` table.
	 */
	type_id: number;
}
export interface PostgresTypeAttribute {
	name: string;
	type_id: number;
}
