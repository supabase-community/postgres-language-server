//! The changes that statements are made of: defining, moving, and dropping relations, types,
//! functions, and schemas in the overlay.

use crate::typing::{FunctionArgumentMode, FunctionSignature, TypeId};
use pgls_query::{Node, protobuf};

use super::{
    Catalog, Entry, Key, key,
    names::{QualifiedName, qualified_name, range_var_name, string_value, type_label},
};
use crate::lookup::{
    CatalogView, ColumnInfo, FunctionInfo, FunctionKind, Lookup, Origin, RelationInfo,
    RelationKind, TypeInfo,
};

impl Catalog {
    // ----- names of new objects -----

    /// The schema an unqualified object is created in: the first existing schema of the search
    /// path.
    pub(super) fn creation_schema(&self, schema: Option<&str>, search_path: &[String]) -> String {
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

    pub(super) fn creation_key(&self, name: &QualifiedName, search_path: &[String]) -> Key {
        (
            self.creation_schema(name.schema(), search_path),
            name.name.clone(),
        )
    }

    /// The key of a new relation. Temporary relations live in `pg_temp`.
    pub(super) fn relation_creation_key(
        &self,
        range_var: &protobuf::RangeVar,
        search_path: &[String],
    ) -> Key {
        let name = range_var_name(range_var);
        if range_var.relpersistence == "t" {
            return key("pg_temp", &name.name);
        }
        self.creation_key(&name, search_path)
    }

    // ----- schemas -----

    pub(super) fn drop_schema(&mut self, name: &str) {
        self.schemas.insert(name.to_owned(), false);
        self.dropped_schemas.insert(name.to_owned());
        self.relations.retain(|(schema, _), _| schema != name);
        self.types.retain(|(schema, _), _| schema != name);
        self.functions.retain(|(schema, _), _| schema != name);
    }

    // ----- relations -----

    pub(super) fn relation_of(
        &self,
        range_var: &protobuf::RangeVar,
        search_path: &[String],
    ) -> Lookup<RelationInfo> {
        let name = range_var_name(range_var);
        self.relation(name.schema(), &name.name, search_path)
    }

    /// Stores a relation and its row type.
    pub(super) fn define_relation(&mut self, relation: RelationInfo) {
        let key = key(&relation.schema, &relation.name);
        if relation.kind != RelationKind::Other {
            self.types.insert(
                key.clone(),
                Entry::Defined(TypeInfo {
                    schema: relation.schema.clone(),
                    name: relation.name.clone(),
                    attributes: relation.columns.clone(),
                    origin: Origin::File,

                    id: Some(
                        self.types
                            .get(&key)
                            .and_then(|entry| match entry {
                                Entry::Defined(existing) => existing.id.clone(),
                                _ => None,
                            })
                            .unwrap_or_else(|| self.allocate_type_id()),
                    ),
                    kind: Some(crate::typing::TypeKind::Composite),
                    category: Some('C'),
                    preferred: Some(false),
                    element: None,
                    array: None,
                    base: None,
                    relation: None,
                }),
            );
            self.ensure_array_type(&relation.schema, &relation.name, None);
        }
        self.relations.insert(key, Entry::Defined(relation));
    }

    pub(super) fn mark_relation_unknown(&mut self, key: Key) {
        self.types.insert(key.clone(), Entry::Unknown);
        self.relations.insert(key, Entry::Unknown);
    }

    /// Handles `IF NOT EXISTS` for a new relation. Returns `true` if the statement is a no-op.
    pub(super) fn skip_existing_relation(&mut self, key: &Key, if_not_exists: bool) -> bool {
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

    /// Changes the columns of a relation and of its inheritance children and partitions.
    pub(super) fn change_columns(
        &mut self,
        relation: &RelationInfo,
        change: impl Fn(&mut Option<Vec<ColumnInfo>>),
    ) {
        // Column changes of a database table also change its children in the database.
        let is_database_table = self
            .base
            .as_ref()
            .is_some_and(|base| base.relation(&relation.schema, &relation.name).is_some());
        if is_database_table {
            self.database_columns_changed = true;
        }

        let mut pending = vec![key(&relation.schema, &relation.name)];
        while let Some(key) = pending.pop() {
            if let Some(children) = self.children.get(&key) {
                pending.extend(children.iter().cloned());
            }
            let Some(mut relation) = self.relation_in(&key.0, &key.1).found() else {
                continue;
            };
            change(&mut relation.columns);
            relation.origin = Origin::File;
            self.define_relation(relation);
        }
    }

    /// Moves a relation to a new schema or name.
    pub(super) fn move_relation(
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
            let old_type = self.types.get(&old_key).and_then(|entry| match entry {
                Entry::Defined(info) => Some(info.clone()),
                _ => None,
            });
            self.types.insert(old_key.clone(), Entry::Dropped);
            if let Some(mut info) = old_type {
                let old_array = key(&info.schema, &format!("_{}", info.name));
                let array_info = self.types.get(&old_array).and_then(|entry| match entry {
                    Entry::Defined(info) => Some(info.clone()),
                    _ => None,
                });
                self.types.insert(old_array, Entry::Dropped);
                info.schema = relation.schema.clone();
                info.name = relation.name.clone();
                self.types
                    .insert(key(&info.schema, &info.name), Entry::Defined(info.clone()));
                if let Some(mut array) = array_info {
                    array.schema = relation.schema.clone();
                    array.name = format!("_{}", relation.name);
                    self.types
                        .insert(key(&array.schema, &array.name), Entry::Defined(array));
                }
            }
        }
        if let Some(children) = self.children.remove(&old_key) {
            self.children
                .insert(key(&relation.schema, &relation.name), children);
        }
        self.define_relation(relation);
    }

    // ----- types -----

    pub(super) fn define_type(
        &mut self,
        name: &QualifiedName,
        attributes: Option<Vec<ColumnInfo>>,
        search_path: &[String],
    ) {
        let (schema, name) = self.creation_key(name, search_path);
        self.define_type_with_metadata(
            schema,
            name,
            attributes,
            crate::typing::TypeKind::Composite,
            'C',
            None,
        );
    }

    pub(super) fn define_type_with_metadata(
        &mut self,
        schema: String,
        name: String,
        attributes: Option<Vec<ColumnInfo>>,
        kind: crate::typing::TypeKind,
        category: char,
        base: Option<crate::typing::TypeId>,
    ) {
        let type_key = key(&schema, &name);
        let id = self.allocate_type_id();
        let array_name = format!("_{name}");
        let array_id = self.allocate_type_id();
        self.types.insert(
            type_key,
            Entry::Defined(TypeInfo {
                schema: schema.clone(),
                name: name.clone(),
                attributes: attributes.clone(),
                origin: Origin::File,
                id: Some(id.clone()),
                kind: Some(kind),
                category: Some(category),
                preferred: Some(false),
                element: None,
                array: Some(array_id.clone()),
                base,
                relation: None,
            }),
        );
        self.types.insert(
            key(&schema, &array_name),
            Entry::Defined(TypeInfo {
                schema,
                name: array_name,
                attributes: None,
                origin: Origin::File,
                id: Some(array_id),
                kind: Some(crate::typing::TypeKind::Base),
                category: Some('A'),
                preferred: Some(false),
                element: Some(id),
                array: None,
                base: None,
                relation: None,
            }),
        );
    }

    fn ensure_array_type(&mut self, schema: &str, name: &str, attributes: Option<Vec<ColumnInfo>>) {
        let row_key = key(schema, name);
        let row_id = self.types.get(&row_key).and_then(|entry| match entry {
            Entry::Defined(info) => info.id.clone(),
            _ => None,
        });
        let Some(row_id) = row_id else {
            return;
        };
        let array_name = format!("_{name}");
        let array_key = key(schema, &array_name);
        let array_id = self
            .types
            .get(&array_key)
            .and_then(|entry| match entry {
                Entry::Defined(info) => info.id.clone(),
                _ => None,
            })
            .unwrap_or_else(|| self.allocate_type_id());
        if let Some(Entry::Defined(row)) = self.types.get_mut(&row_key) {
            row.array = Some(array_id.clone());
        }
        self.types.insert(
            array_key,
            Entry::Defined(TypeInfo {
                schema: schema.to_owned(),
                name: array_name,
                attributes,
                origin: Origin::File,
                id: Some(array_id),
                kind: Some(crate::typing::TypeKind::Base),
                category: Some('A'),
                preferred: Some(false),
                element: Some(row_id),
                array: None,
                base: None,
                relation: None,
            }),
        );
    }

    /// Moves a type to a new schema or name.
    pub(super) fn move_type(
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

    // ----- functions -----

    /// Adds an overload to the functions with its name.
    pub(super) fn add_function(&mut self, function: FunctionInfo) {
        let key = key(&function.schema, &function.name);
        let entry = match self.functions_in(&key.0, &key.1) {
            Lookup::Found(mut overloads) => {
                // `CREATE OR REPLACE` replaces the overload with the same input types. When
                // the types are not all known, we can't tell, so the overload is added.
                let replaced = input_types(function.signature.as_ref()).and_then(|types| {
                    overloads.iter().position(|existing| {
                        input_types(existing.signature.as_ref()).as_ref() == Some(&types)
                    })
                });
                match replaced {
                    Some(position) => overloads[position] = function,
                    None => overloads.push(function),
                }
                Entry::Defined(overloads)
            }
            Lookup::Missing => Entry::Defined(vec![function]),
            Lookup::Unknown => Entry::Unknown,
        };
        self.functions.insert(key, entry);
    }

    /// Removes the overload of `object` from its function set and returns it, if it can be
    /// identified. If it can't, the function set becomes unknown.
    pub(super) fn take_function(
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
    pub(super) fn move_function(
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
}

pub(super) fn column_info(
    catalog: &Catalog,
    column: &protobuf::ColumnDef,
    search_path: &[String],
) -> ColumnInfo {
    let type_id = column
        .type_name
        .as_ref()
        .and_then(|name| crate::normalize_type_name(catalog, name, search_path).0);
    let type_name = column.type_name.as_ref().and_then(type_label);
    ColumnInfo {
        name: column.colname.clone(),
        type_name,
        ty: type_id.map(crate::typing::Type::Named),
    }
}

pub(super) fn function_info(
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
        signature: None,
    }
}

/// Applies explicit column names (`CREATE VIEW v (a, b)`, `CREATE TABLE t (a, b) AS`). Columns
/// without an explicit name keep their derived name.
pub(super) fn rename_columns(
    columns: Option<Vec<ColumnInfo>>,
    names: &[Node],
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

/// The input argument types of a function, or `None` if any of them is unknown.
fn input_types(signature: Option<&FunctionSignature>) -> Option<Vec<&TypeId>> {
    signature?
        .arguments
        .iter()
        .filter(|arg| {
            matches!(
                arg.mode,
                FunctionArgumentMode::In
                    | FunctionArgumentMode::InOut
                    | FunctionArgumentMode::Variadic
            )
        })
        .map(|arg| arg.ty.as_ref())
        .collect()
}
