//! Applying the effects of statements to the catalog.
//!
//! Only DDL that creates, alters, renames, or drops relations, columns, types, functions, or
//! schemas is modelled. Statements that may change these in ways we can't follow taint the
//! catalog. Everything else (DML, `SET`, transactions, indexes, grants, comments, policies,
//! triggers, constraints, ...) has no effect on name resolution and is ignored.
//!
//! Known gaps, all of which can only hide errors, never report wrong ones:
//! - `CASCADE` does not drop dependent objects.
//! - `DROP EXTENSION` does not drop the extension's objects.

use pgls_query::{NodeEnum, protobuf};
use protobuf::ObjectType;

use super::{
    Catalog, Entry, Key, Snapshot,
    base::sequence_columns,
    key,
    names::{QualifiedName, qualified_name, range_var_name, string_value, type_label, type_name},
};
use crate::view::{
    CatalogView, ColumnInfo, FunctionInfo, FunctionKind, Lookup, Origin, RelationInfo,
    RelationKind, TypeInfo,
};

impl Catalog {
    /// Applies the effects of a top-level statement, executed with the given search path.
    pub fn apply(&mut self, stmt: &NodeEnum, search_path: &[String]) {
        match stmt {
            NodeEnum::CreateSchemaStmt(stmt) => self.create_schema(stmt, search_path),
            NodeEnum::CreateStmt(stmt) => self.create_table(stmt, RelationKind::Table, search_path),
            NodeEnum::CreateForeignTableStmt(stmt) => {
                if let Some(base) = &stmt.base_stmt {
                    self.create_table(base, RelationKind::ForeignTable, search_path);
                }
            }
            NodeEnum::CreateTableAsStmt(stmt) => self.create_table_as(stmt, search_path),
            NodeEnum::SelectStmt(stmt) => {
                if let Some(into) = &stmt.into_clause {
                    let columns = self.select_columns_for_into(stmt, search_path);
                    self.create_from_into(into, RelationKind::Table, columns, false, search_path);
                }
            }
            NodeEnum::ViewStmt(stmt) => self.create_view(stmt, search_path),
            NodeEnum::CreateSeqStmt(stmt) => self.create_sequence(stmt, search_path),
            NodeEnum::CompositeTypeStmt(stmt) => self.create_composite_type(stmt, search_path),
            NodeEnum::CreateEnumStmt(stmt) => {
                if let Some(name) = qualified_name(&stmt.type_name) {
                    self.create_type(&name, None, search_path);
                }
            }
            NodeEnum::CreateDomainStmt(stmt) => {
                if let Some(name) = qualified_name(&stmt.domainname) {
                    self.create_type(&name, None, search_path);
                }
            }
            NodeEnum::CreateRangeStmt(stmt) => self.create_range_type(stmt, search_path),
            NodeEnum::DefineStmt(stmt) => self.define(stmt, search_path),
            NodeEnum::CreateFunctionStmt(stmt) => self.create_function(stmt, search_path),
            NodeEnum::AlterTableStmt(stmt) => self.alter_table(stmt, search_path),
            NodeEnum::RenameStmt(stmt) => self.rename(stmt, search_path),
            NodeEnum::AlterObjectSchemaStmt(stmt) => self.set_schema(stmt, search_path),
            NodeEnum::DropStmt(stmt) => self.drop(stmt, search_path),
            NodeEnum::TransactionStmt(stmt) => self.transaction(stmt),
            NodeEnum::CreateExtensionStmt(stmt) => {
                let installed = self
                    .base
                    .as_ref()
                    .is_some_and(|base| base.has_installed_extension(&stmt.extname));
                if !installed {
                    self.tainted = true;
                }
            }
            // These run arbitrary code or create objects we can't see.
            NodeEnum::DoStmt(_)
            | NodeEnum::CallStmt(_)
            | NodeEnum::ExecuteStmt(_)
            | NodeEnum::ImportForeignSchemaStmt(_)
            | NodeEnum::AlterExtensionStmt(_)
            | NodeEnum::AlterExtensionContentsStmt(_) => self.tainted = true,
            _ => {}
        }
    }

    // ----- transactions -----

    /// DDL is transactional: `ROLLBACK` restores the catalog of `BEGIN` or the savepoint.
    fn transaction(&mut self, stmt: &protobuf::TransactionStmt) {
        use protobuf::TransactionStmtKind as Kind;
        match stmt.kind() {
            Kind::TransStmtBegin | Kind::TransStmtStart => {
                // A nested BEGIN only warns.
                if self.snapshots.is_empty() {
                    self.push_snapshot(None);
                }
            }
            Kind::TransStmtSavepoint => self.push_snapshot(Some(stmt.savepoint_name.clone())),
            Kind::TransStmtRelease => {
                if let Some(position) = self.savepoint_position(&stmt.savepoint_name) {
                    self.snapshots.truncate(position);
                }
            }
            Kind::TransStmtRollbackTo => {
                if let Some(position) = self.savepoint_position(&stmt.savepoint_name) {
                    let snapshots = self.snapshots[..=position].to_vec();
                    *self = *snapshots[position].catalog.clone();
                    self.snapshots = snapshots;
                }
            }
            Kind::TransStmtRollback | Kind::TransStmtRollbackPrepared => {
                if let Some(snapshot) = self.snapshots.first().cloned() {
                    *self = *snapshot.catalog;
                }
            }
            Kind::TransStmtCommit | Kind::TransStmtCommitPrepared | Kind::TransStmtPrepare => {
                self.snapshots.clear();
            }
            _ => {}
        }
    }

    fn push_snapshot(&mut self, savepoint: Option<String>) {
        let mut catalog = self.clone();
        catalog.snapshots.clear();
        self.snapshots.push(Snapshot {
            savepoint,
            catalog: Box::new(catalog),
        });
    }

    fn savepoint_position(&self, name: &str) -> Option<usize> {
        self.snapshots
            .iter()
            .rposition(|snapshot| snapshot.savepoint.as_deref() == Some(name))
    }

    // ----- schemas -----

    fn create_schema(&mut self, stmt: &protobuf::CreateSchemaStmt, search_path: &[String]) {
        let name = if !stmt.schemaname.is_empty() {
            stmt.schemaname.clone()
        } else {
            // `CREATE SCHEMA AUTHORIZATION role` names the schema after the role.
            match stmt.authrole.as_ref() {
                Some(role)
                    if role.roletype() == protobuf::RoleSpecType::RolespecCstring
                        && !role.rolename.is_empty() =>
                {
                    role.rolename.clone()
                }
                _ => {
                    self.tainted = true;
                    return;
                }
            }
        };

        if stmt.if_not_exists && self.schema(&name).found().is_some() {
            return;
        }
        self.schemas.insert(name.clone(), true);

        // Schema elements are created in the new schema, which is searched first.
        let element_path: Vec<String> = std::iter::once(name)
            .chain(search_path.iter().cloned())
            .collect();
        for element in &stmt.schema_elts {
            if let Some(element) = &element.node {
                self.apply(element, &element_path);
            }
        }
    }

    fn drop_schema(&mut self, name: &str) {
        self.schemas.insert(name.to_owned(), false);
        self.dropped_schemas.insert(name.to_owned());
        self.relations.retain(|(schema, _), _| schema != name);
        self.types.retain(|(schema, _), _| schema != name);
        self.functions.retain(|(schema, _), _| schema != name);
    }

    /// The schema an unqualified object is created in: the first existing schema of the search
    /// path.
    fn creation_schema(&self, schema: Option<&str>, search_path: &[String]) -> String {
        if let Some(schema) = schema {
            return schema.to_owned();
        }
        search_path
            .iter()
            .filter(|schema| schema.as_str() != "$user")
            .find(|schema| !self.schema(schema).is_missing())
            .cloned()
            .unwrap_or_else(|| "public".into())
    }

    fn creation_key(&self, name: &QualifiedName, search_path: &[String]) -> Key {
        (
            self.creation_schema(name.schema(), search_path),
            name.name.clone(),
        )
    }

    /// The key of a new relation. Temporary relations live in `pg_temp`.
    fn relation_creation_key(&self, range_var: &protobuf::RangeVar, search_path: &[String]) -> Key {
        let name = range_var_name(range_var);
        if range_var.relpersistence == "t" {
            return key("pg_temp", &name.name);
        }
        self.creation_key(&name, search_path)
    }

    // ----- relations -----

    /// Stores a relation and its row type.
    fn define_relation(&mut self, relation: RelationInfo) {
        let key = key(&relation.schema, &relation.name);
        if relation.kind != RelationKind::Other {
            self.types.insert(
                key.clone(),
                Entry::Defined(TypeInfo {
                    schema: relation.schema.clone(),
                    name: relation.name.clone(),
                    attributes: relation.columns.clone(),
                    origin: Origin::File,
                }),
            );
        }
        self.relations.insert(key, Entry::Defined(relation));
    }

    fn mark_relation_unknown(&mut self, key: Key) {
        self.types.insert(key.clone(), Entry::Unknown);
        self.relations.insert(key, Entry::Unknown);
    }

    /// Handles `IF NOT EXISTS` for a new relation. Returns `true` if the statement is a no-op.
    fn skip_existing_relation(&mut self, key: &Key, if_not_exists: bool) -> bool {
        if !if_not_exists {
            return false;
        }
        match self.relation_in(&key.0, &key.1) {
            Lookup::Found(_) => true,
            Lookup::Unknown => {
                self.mark_relation_unknown(key.clone());
                true
            }
            Lookup::Missing => false,
        }
    }

    fn relation_of(
        &self,
        range_var: &protobuf::RangeVar,
        search_path: &[String],
    ) -> Lookup<RelationInfo> {
        let name = range_var_name(range_var);
        self.relation(name.schema(), &name.name, search_path)
    }

    fn create_table(
        &mut self,
        stmt: &protobuf::CreateStmt,
        kind: RelationKind,
        search_path: &[String],
    ) {
        let Some(range_var) = &stmt.relation else {
            return;
        };
        let key = self.relation_creation_key(range_var, search_path);
        if self.skip_existing_relation(&key, stmt.if_not_exists) {
            return;
        }

        let mut columns: Option<Vec<ColumnInfo>> = Some(Vec::new());
        let mut add_columns = |new: Option<Vec<ColumnInfo>>| match (&mut columns, new) {
            (Some(columns), Some(new)) => {
                for column in new {
                    if !columns.iter().any(|c: &ColumnInfo| c.name == column.name) {
                        columns.push(column);
                    }
                }
            }
            _ => columns = None,
        };

        // Typed tables (`OF type`) take the attributes of the type.
        if let Some(of_type) = &stmt.of_typename {
            let attributes = type_name(of_type).and_then(|name| {
                self.type_(name.schema(), &name.name, search_path)
                    .found()
                    .and_then(|type_info| type_info.attributes)
            });
            add_columns(attributes);
        }

        // Inherited columns (`INHERITS`, `PARTITION OF`) come first.
        let mut parents = Vec::new();
        for parent in &stmt.inh_relations {
            let parent = match parent.node.as_ref() {
                Some(NodeEnum::RangeVar(parent)) => self.relation_of(parent, search_path).found(),
                _ => None,
            };
            if let Some(parent) = &parent {
                parents.push(super::key(&parent.schema, &parent.name));
            }
            add_columns(parent.and_then(|parent| parent.columns));
        }

        for element in &stmt.table_elts {
            match element.node.as_ref() {
                Some(NodeEnum::ColumnDef(column)) => add_columns(Some(vec![column_info(column)])),
                Some(NodeEnum::TableLikeClause(like)) => {
                    let source = like
                        .relation
                        .as_ref()
                        .and_then(|source| self.relation_of(source, search_path).found());
                    add_columns(source.and_then(|source| source.columns));
                }
                _ => {}
            }
        }

        for parent in parents {
            self.children.entry(parent).or_default().push(key.clone());
        }

        let kind = if stmt.partspec.is_some() {
            RelationKind::PartitionedTable
        } else {
            kind
        };
        self.define_relation(RelationInfo {
            schema: key.0,
            name: key.1,
            kind,
            columns,
            origin: Origin::File,
        });
    }

    fn create_table_as(&mut self, stmt: &protobuf::CreateTableAsStmt, search_path: &[String]) {
        let Some(into) = &stmt.into else {
            return;
        };
        let kind = if stmt.objtype() == ObjectType::ObjectMatview {
            RelationKind::MaterializedView
        } else {
            RelationKind::Table
        };
        let columns = stmt
            .query
            .as_deref()
            .and_then(|query| self.query_columns(query, search_path));
        self.create_from_into(into, kind, columns, stmt.if_not_exists, search_path);
    }

    fn select_columns_for_into(
        &self,
        stmt: &protobuf::SelectStmt,
        search_path: &[String],
    ) -> Option<Vec<ColumnInfo>> {
        let mut query = stmt.clone();
        query.into_clause = None;
        let query = protobuf::Node {
            node: Some(NodeEnum::SelectStmt(Box::new(query))),
        };
        self.query_columns(&query, search_path)
    }

    /// Creates the relation of `CREATE TABLE AS`, `CREATE MATERIALIZED VIEW` or `SELECT INTO`.
    fn create_from_into(
        &mut self,
        into: &protobuf::IntoClause,
        kind: RelationKind,
        columns: Option<Vec<ColumnInfo>>,
        if_not_exists: bool,
        search_path: &[String],
    ) {
        let Some(range_var) = &into.rel else {
            return;
        };
        let key = self.relation_creation_key(range_var, search_path);
        if self.skip_existing_relation(&key, if_not_exists) {
            return;
        }
        let columns = rename_columns(columns, &into.col_names);
        self.define_relation(RelationInfo {
            schema: key.0,
            name: key.1,
            kind,
            columns,
            origin: Origin::File,
        });
    }

    fn create_view(&mut self, stmt: &protobuf::ViewStmt, search_path: &[String]) {
        let Some(range_var) = &stmt.view else {
            return;
        };
        let key = self.relation_creation_key(range_var, search_path);
        let columns = stmt
            .query
            .as_deref()
            .and_then(|query| self.query_columns(query, search_path));
        let columns = rename_columns(columns, &stmt.aliases);
        self.define_relation(RelationInfo {
            schema: key.0,
            name: key.1,
            kind: RelationKind::View,
            columns,
            origin: Origin::File,
        });
    }

    fn create_sequence(&mut self, stmt: &protobuf::CreateSeqStmt, search_path: &[String]) {
        let Some(range_var) = &stmt.sequence else {
            return;
        };
        let key = self.relation_creation_key(range_var, search_path);
        if self.skip_existing_relation(&key, stmt.if_not_exists) {
            return;
        }
        self.define_relation(RelationInfo {
            schema: key.0,
            name: key.1,
            kind: RelationKind::Other,
            columns: Some(sequence_columns()),
            origin: Origin::File,
        });
    }

    fn alter_table(&mut self, stmt: &protobuf::AlterTableStmt, search_path: &[String]) {
        let Some(range_var) = &stmt.relation else {
            return;
        };

        // `ALTER TYPE ... ADD/DROP/ALTER ATTRIBUTE` on composite types.
        if stmt.objtype() == ObjectType::ObjectType {
            let name = range_var_name(range_var);
            if let Some(mut type_info) = self.type_(name.schema(), &name.name, search_path).found()
            {
                if let Some(attributes) = type_info.attributes.as_mut() {
                    for command in &stmt.cmds {
                        if let Some(NodeEnum::AlterTableCmd(command)) = &command.node {
                            apply_column_change(attributes, command);
                        }
                    }
                }
                type_info.origin = Origin::File;
                let key = key(&type_info.schema, &type_info.name);
                self.types.insert(key, Entry::Defined(type_info));
            }
            return;
        }

        let Some(relation) = self.relation_of(range_var, search_path).found() else {
            return;
        };
        let commands: Vec<&protobuf::AlterTableCmd> = stmt
            .cmds
            .iter()
            .filter_map(|command| match &command.node {
                Some(NodeEnum::AlterTableCmd(command)) => Some(command.as_ref()),
                _ => None,
            })
            .collect();
        let changes_columns = commands.iter().any(|command| {
            matches!(
                command.subtype(),
                protobuf::AlterTableType::AtAddColumn
                    | protobuf::AlterTableType::AtDropColumn
                    | protobuf::AlterTableType::AtAlterColumnType
            )
        });
        if !changes_columns {
            return;
        }

        self.note_column_change(&relation);

        // Column changes also apply to inheritance children and partitions.
        let mut pending = vec![key(&relation.schema, &relation.name)];
        while let Some(key) = pending.pop() {
            if let Some(children) = self.children.get(&key) {
                pending.extend(children.iter().cloned());
            }
            let Some(mut relation) = self.relation_in(&key.0, &key.1).found() else {
                continue;
            };
            if let Some(columns) = relation.columns.as_mut() {
                for command in &commands {
                    apply_column_change(columns, command);
                }
            }
            relation.origin = Origin::File;
            self.define_relation(relation);
        }
    }

    /// Column changes of a database table also change its children in the database.
    fn note_column_change(&mut self, relation: &RelationInfo) {
        let is_database_table = self
            .base
            .as_ref()
            .is_some_and(|base| base.relation(&relation.schema, &relation.name).is_some());
        if is_database_table {
            self.database_columns_changed = true;
        }
    }

    // ----- types -----

    fn create_type(
        &mut self,
        name: &QualifiedName,
        attributes: Option<Vec<ColumnInfo>>,
        search_path: &[String],
    ) {
        let (schema, name) = self.creation_key(name, search_path);
        self.types.insert(
            key(&schema, &name),
            Entry::Defined(TypeInfo {
                schema,
                name,
                attributes,
                origin: Origin::File,
            }),
        );
    }

    fn create_composite_type(
        &mut self,
        stmt: &protobuf::CompositeTypeStmt,
        search_path: &[String],
    ) {
        let Some(range_var) = &stmt.typevar else {
            return;
        };
        let attributes = stmt
            .coldeflist
            .iter()
            .filter_map(|node| match &node.node {
                Some(NodeEnum::ColumnDef(column)) => Some(column_info(column)),
                _ => None,
            })
            .collect();
        let name = range_var_name(range_var);
        let key = self.creation_key(&name, search_path);
        self.create_type(&name, Some(attributes), search_path);
        // Composite types are relations, too, but they can't be queried.
        self.relations.insert(key, Entry::Unknown);
    }

    fn create_range_type(&mut self, stmt: &protobuf::CreateRangeStmt, search_path: &[String]) {
        let Some(name) = qualified_name(&stmt.type_name) else {
            return;
        };
        let custom_multirange_name = stmt.params.iter().any(|param| {
            matches!(&param.node, Some(NodeEnum::DefElem(param)) if param.defname == "multirange_type_name")
        });
        if custom_multirange_name {
            self.tainted = true;
            return;
        }

        let (schema, range_name) = self.creation_key(&name, search_path);
        let multirange_name = if range_name.contains("range") {
            range_name.replacen("range", "multirange", 1)
        } else {
            format!("{range_name}_multirange")
        };

        for type_name in [&range_name, &multirange_name] {
            self.create_type(
                &QualifiedName {
                    schema: Some(schema.clone()),
                    name: type_name.clone(),
                },
                None,
                search_path,
            );
        }
        // `range(lower, upper [, bounds])` and `multirange(VARIADIC ranges)`.
        self.add_function(function_info(
            &schema,
            &range_name,
            FunctionKind::Function,
            2,
            Some(3),
        ));
        self.add_function(function_info(
            &schema,
            &multirange_name,
            FunctionKind::Function,
            0,
            None,
        ));
    }

    /// `CREATE AGGREGATE`, `CREATE TYPE name (...)` and shell types.
    fn define(&mut self, stmt: &protobuf::DefineStmt, search_path: &[String]) {
        let Some(name) = qualified_name(&stmt.defnames) else {
            return;
        };
        match stmt.kind() {
            ObjectType::ObjectType => self.create_type(&name, None, search_path),
            ObjectType::ObjectAggregate => {
                let (schema, name) = self.creation_key(&name, search_path);
                // The arity of aggregates (ordered-set, `*`, variadic) is not modelled.
                self.add_function(function_info(
                    &schema,
                    &name,
                    FunctionKind::Aggregate,
                    0,
                    None,
                ));
            }
            _ => {}
        }
    }

    // ----- functions -----

    fn create_function(&mut self, stmt: &protobuf::CreateFunctionStmt, search_path: &[String]) {
        let Some(name) = qualified_name(&stmt.funcname) else {
            return;
        };
        let (schema, name) = self.creation_key(&name, search_path);

        let mut inputs = 0;
        let mut defaults = 0;
        let mut variadic = false;
        let mut outputs = Vec::new();
        for parameter in &stmt.parameters {
            let Some(NodeEnum::FunctionParameter(parameter)) = &parameter.node else {
                continue;
            };
            use protobuf::FunctionParameterMode as Mode;
            let mode = parameter.mode();
            if matches!(
                mode,
                Mode::FuncParamOut | Mode::FuncParamInout | Mode::FuncParamTable
            ) {
                outputs.push(ColumnInfo {
                    name: parameter.name.clone(),
                    type_name: parameter.arg_type.as_ref().and_then(type_label),
                });
            }
            if matches!(mode, Mode::FuncParamOut | Mode::FuncParamTable) {
                continue;
            }
            inputs += 1;
            variadic |= mode == Mode::FuncParamVariadic;
            if parameter.defexpr.is_some() {
                defaults += 1;
            }
        }

        let return_type = stmt.return_type.as_ref();
        let returns_set = return_type.is_some_and(|return_type| return_type.setof);
        let return_columns = if outputs.is_empty() {
            return_type
                .and_then(type_name)
                .and_then(|name| self.type_(name.schema(), &name.name, search_path).found())
                .and_then(|type_info| type_info.attributes)
        } else {
            Some(outputs)
        };

        let kind = if stmt.is_procedure {
            FunctionKind::Procedure
        } else {
            FunctionKind::Function
        };
        let mut function = function_info(
            &schema,
            &name,
            kind,
            inputs - defaults,
            (!variadic).then_some(inputs),
        );
        function.returns_set = returns_set;
        function.return_columns = return_columns;
        self.add_function(function);
    }

    /// Adds an overload to the functions with its name.
    fn add_function(&mut self, function: FunctionInfo) {
        let key = key(&function.schema, &function.name);
        let entry = match self.functions_in(&key.0, &key.1) {
            Lookup::Found(mut overloads) => {
                overloads.push(function);
                Entry::Defined(overloads)
            }
            Lookup::Missing => Entry::Defined(vec![function]),
            Lookup::Unknown => Entry::Unknown,
        };
        self.functions.insert(key, entry);
    }

    /// Removes the overload of `object` from its function set and returns it, if it can be
    /// identified. If it can't, the function set becomes unknown.
    fn take_function(
        &mut self,
        object: &protobuf::ObjectWithArgs,
        search_path: &[String],
    ) -> Option<FunctionInfo> {
        let name = qualified_name(&object.objname)?;
        let key = self.function_key(name.schema(), &name.name, search_path)?;
        let Lookup::Found(mut overloads) = self.functions_in(&key.0, &key.1) else {
            return None;
        };

        let arg_count = object.objargs.len();
        let position = if object.args_unspecified || overloads.len() == 1 {
            (overloads.len() == 1).then_some(0)
        } else if overloads.iter().any(|overload| overload.max_args.is_none()) {
            None
        } else {
            let mut candidates = overloads
                .iter()
                .enumerate()
                .filter(|(_, overload)| overload.max_args == Some(arg_count));
            match (candidates.next(), candidates.next()) {
                (Some((position, _)), None) => Some(position),
                _ => None,
            }
        };

        let Some(position) = position else {
            self.functions.insert(key, Entry::Unknown);
            return None;
        };
        let function = overloads.remove(position);
        let entry = if overloads.is_empty() {
            Entry::Dropped
        } else {
            Entry::Defined(overloads)
        };
        self.functions.insert(key, entry);
        Some(function)
    }

    /// Moves a function to a new schema or name. If the function can't be identified, the
    /// target becomes unknown.
    fn move_function(
        &mut self,
        object: &protobuf::ObjectWithArgs,
        new_schema: Option<&str>,
        new_name: Option<&str>,
        search_path: &[String],
    ) {
        let function = self.take_function(object, search_path);
        match function {
            Some(mut function) => {
                if let Some(schema) = new_schema {
                    function.schema = schema.to_owned();
                }
                if let Some(name) = new_name {
                    function.name = name.to_owned();
                }
                function.origin = Origin::File;
                self.add_function(function);
            }
            None => {
                let Some(name) = qualified_name(&object.objname) else {
                    return;
                };
                let schema = new_schema
                    .map(str::to_owned)
                    .or(name.schema)
                    .unwrap_or_else(|| self.creation_schema(None, search_path));
                let name = new_name.map_or(name.name, str::to_owned);
                self.functions.insert((schema, name), Entry::Unknown);
            }
        }
    }

    // ----- renames and moves -----

    fn rename(&mut self, stmt: &protobuf::RenameStmt, search_path: &[String]) {
        let object = stmt
            .object
            .as_deref()
            .and_then(|object| object.node.as_ref());
        match stmt.rename_type() {
            ObjectType::ObjectTable
            | ObjectType::ObjectView
            | ObjectType::ObjectMatview
            | ObjectType::ObjectForeignTable
            | ObjectType::ObjectSequence => {
                if let Some(range_var) = &stmt.relation {
                    self.move_relation(range_var, None, Some(&stmt.newname), search_path);
                }
            }
            ObjectType::ObjectColumn => {
                if let Some(range_var) = &stmt.relation {
                    self.rename_column(range_var, &stmt.subname, &stmt.newname, search_path);
                }
            }
            ObjectType::ObjectAttribute => {
                if let Some(range_var) = &stmt.relation {
                    let name = range_var_name(range_var);
                    if let Some(mut type_info) =
                        self.type_(name.schema(), &name.name, search_path).found()
                    {
                        rename_in(&mut type_info.attributes, &stmt.subname, &stmt.newname);
                        type_info.origin = Origin::File;
                        let key = key(&type_info.schema, &type_info.name);
                        self.types.insert(key, Entry::Defined(type_info));
                    }
                }
            }
            ObjectType::ObjectType | ObjectType::ObjectDomain => {
                if let Some(NodeEnum::List(list)) = object {
                    if let Some(name) = qualified_name(&list.items) {
                        self.move_type(&name, None, Some(&stmt.newname), search_path);
                    }
                }
            }
            ObjectType::ObjectFunction
            | ObjectType::ObjectProcedure
            | ObjectType::ObjectRoutine
            | ObjectType::ObjectAggregate => {
                if let Some(NodeEnum::ObjectWithArgs(function)) = object {
                    self.move_function(function, None, Some(&stmt.newname), search_path);
                }
            }
            // Moving every object of a schema to a new name is not modelled.
            ObjectType::ObjectSchema => self.tainted = true,
            _ => {}
        }
    }

    fn set_schema(&mut self, stmt: &protobuf::AlterObjectSchemaStmt, search_path: &[String]) {
        let object = stmt
            .object
            .as_deref()
            .and_then(|object| object.node.as_ref());
        let new_schema = stmt.newschema.as_str();
        match stmt.object_type() {
            ObjectType::ObjectTable
            | ObjectType::ObjectView
            | ObjectType::ObjectMatview
            | ObjectType::ObjectForeignTable
            | ObjectType::ObjectSequence => {
                if let Some(range_var) = &stmt.relation {
                    self.move_relation(range_var, Some(new_schema), None, search_path);
                }
            }
            ObjectType::ObjectType | ObjectType::ObjectDomain => {
                if let Some(NodeEnum::List(list)) = object {
                    if let Some(name) = qualified_name(&list.items) {
                        self.move_type(&name, Some(new_schema), None, search_path);
                    }
                }
            }
            ObjectType::ObjectFunction
            | ObjectType::ObjectProcedure
            | ObjectType::ObjectRoutine
            | ObjectType::ObjectAggregate => {
                if let Some(NodeEnum::ObjectWithArgs(function)) = object {
                    self.move_function(function, Some(new_schema), None, search_path);
                }
            }
            // Moves all objects of the extension.
            ObjectType::ObjectExtension => self.tainted = true,
            _ => {}
        }
    }

    fn move_relation(
        &mut self,
        range_var: &protobuf::RangeVar,
        new_schema: Option<&str>,
        new_name: Option<&str>,
        search_path: &[String],
    ) {
        let Some(mut relation) = self.relation_of(range_var, search_path).found() else {
            return;
        };
        let old_key = key(&relation.schema, &relation.name);
        if let Some(schema) = new_schema {
            relation.schema = schema.to_owned();
        }
        if let Some(name) = new_name {
            relation.name = name.to_owned();
        }
        relation.origin = Origin::File;

        self.relations.insert(old_key.clone(), Entry::Dropped);
        if relation.kind != RelationKind::Other {
            self.types.insert(old_key.clone(), Entry::Dropped);
        }
        if let Some(children) = self.children.remove(&old_key) {
            self.children
                .insert(key(&relation.schema, &relation.name), children);
        }
        self.define_relation(relation);
    }

    fn rename_column(
        &mut self,
        range_var: &protobuf::RangeVar,
        old_name: &str,
        new_name: &str,
        search_path: &[String],
    ) {
        let Some(relation) = self.relation_of(range_var, search_path).found() else {
            return;
        };
        self.note_column_change(&relation);
        let mut pending = vec![key(&relation.schema, &relation.name)];
        while let Some(key) = pending.pop() {
            if let Some(children) = self.children.get(&key) {
                pending.extend(children.iter().cloned());
            }
            let Some(mut relation) = self.relation_in(&key.0, &key.1).found() else {
                continue;
            };
            rename_in(&mut relation.columns, old_name, new_name);
            relation.origin = Origin::File;
            self.define_relation(relation);
        }
    }

    fn move_type(
        &mut self,
        name: &QualifiedName,
        new_schema: Option<&str>,
        new_name: Option<&str>,
        search_path: &[String],
    ) {
        let Some(mut type_info) = self.type_(name.schema(), &name.name, search_path).found() else {
            return;
        };
        let old_key = key(&type_info.schema, &type_info.name);
        if let Some(schema) = new_schema {
            type_info.schema = schema.to_owned();
        }
        if let Some(name) = new_name {
            type_info.name = name.to_owned();
        }
        type_info.origin = Origin::File;
        self.types.insert(old_key, Entry::Dropped);
        let new_key = key(&type_info.schema, &type_info.name);
        self.types.insert(new_key, Entry::Defined(type_info));
    }

    // ----- drops -----

    fn drop(&mut self, stmt: &protobuf::DropStmt, search_path: &[String]) {
        for object in &stmt.objects {
            let Some(object) = &object.node else {
                continue;
            };
            match stmt.remove_type() {
                ObjectType::ObjectTable
                | ObjectType::ObjectView
                | ObjectType::ObjectMatview
                | ObjectType::ObjectForeignTable
                | ObjectType::ObjectSequence => {
                    let NodeEnum::List(list) = object else {
                        continue;
                    };
                    let Some(name) = qualified_name(&list.items) else {
                        continue;
                    };
                    if let Some(key) = self.relation_key(name.schema(), &name.name, search_path) {
                        self.types.insert(key.clone(), Entry::Dropped);
                        self.relations.insert(key, Entry::Dropped);
                    }
                }
                ObjectType::ObjectType | ObjectType::ObjectDomain => {
                    let NodeEnum::TypeName(type_) = object else {
                        continue;
                    };
                    let Some(name) = type_name(type_) else {
                        continue;
                    };
                    if let Some(key) = self.type_key(name.schema(), &name.name, search_path) {
                        self.types.insert(key, Entry::Dropped);
                    }
                }
                ObjectType::ObjectFunction
                | ObjectType::ObjectProcedure
                | ObjectType::ObjectRoutine
                | ObjectType::ObjectAggregate => {
                    if let NodeEnum::ObjectWithArgs(function) = object {
                        self.take_function(function, search_path);
                    }
                }
                ObjectType::ObjectSchema => {
                    if let NodeEnum::String(name) = object {
                        self.drop_schema(&name.sval);
                    }
                }
                _ => {}
            }
        }
    }
}

fn column_info(column: &protobuf::ColumnDef) -> ColumnInfo {
    ColumnInfo {
        name: column.colname.clone(),
        type_name: column.type_name.as_ref().and_then(type_label),
    }
}

fn function_info(
    schema: &str,
    name: &str,
    kind: FunctionKind,
    min_args: usize,
    max_args: Option<usize>,
) -> FunctionInfo {
    FunctionInfo {
        schema: schema.to_owned(),
        name: name.to_owned(),
        kind,
        min_args,
        max_args,
        returns_set: false,
        return_columns: None,
        origin: Origin::File,
    }
}

/// Applies explicit column names (`CREATE VIEW v (a, b)`, `CREATE TABLE t (a, b) AS`). Columns
/// without an explicit name keep their derived name.
fn rename_columns(
    columns: Option<Vec<ColumnInfo>>,
    names: &[protobuf::Node],
) -> Option<Vec<ColumnInfo>> {
    let mut columns = columns?;
    if names.len() > columns.len() {
        return None;
    }
    for (column, name) in columns.iter_mut().zip(names) {
        column.name = string_value(name)?.to_owned();
    }
    Some(columns)
}

fn rename_in(columns: &mut Option<Vec<ColumnInfo>>, old_name: &str, new_name: &str) {
    if let Some(column) = columns
        .iter_mut()
        .flatten()
        .find(|column| column.name == old_name)
    {
        column.name = new_name.to_owned();
    }
}

/// Applies `ADD COLUMN`, `DROP COLUMN` and `ALTER COLUMN TYPE` (and their `ATTRIBUTE`
/// counterparts).
fn apply_column_change(columns: &mut Vec<ColumnInfo>, command: &protobuf::AlterTableCmd) {
    use protobuf::AlterTableType as Type;
    match command.subtype() {
        Type::AtAddColumn => {
            if let Some(NodeEnum::ColumnDef(column)) =
                command.def.as_deref().and_then(|def| def.node.as_ref())
            {
                if !columns.iter().any(|c| c.name == column.colname) {
                    columns.push(column_info(column));
                }
            }
        }
        Type::AtDropColumn => columns.retain(|column| column.name != command.name),
        Type::AtAlterColumnType => {
            if let (Some(NodeEnum::ColumnDef(definition)), Some(column)) = (
                command.def.as_deref().and_then(|def| def.node.as_ref()),
                columns
                    .iter_mut()
                    .find(|column| column.name == command.name),
            ) {
                column.type_name = definition.type_name.as_ref().and_then(type_label);
            }
        }
        _ => {}
    }
}
