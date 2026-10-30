//! Roles, and schemas named on their own rather than as the qualifier of a name, like
//! `grant usage on schema app to reader` or `set search_path to app`. They are only collected as
//! references, and never reported: the catalog doesn't follow the roles a file creates or drops.

use pgls_query::{
    NodeEnum, NodeRef,
    protobuf::{AConst, GrantTargetType, ObjectType, RoleSpecType, Token, a_const},
};

use super::{Reference, resolver::Resolver};
use crate::lookup::Lookup;

/// Records the roles and the schemas named on their own in `stmt`, when references are being
/// collected.
pub(super) fn record_roles_and_schemas(r: &mut Resolver, stmt: &NodeEnum) {
    if !r.collects_references() {
        return;
    }
    for node in stmt.iter() {
        match node {
            NodeRef::RoleSpec(spec) if spec.roletype() == RoleSpecType::RolespecCstring => {
                record_role(r, &spec.rolename, spec.location);
            }
            // `set search_path to app, public`, `set role reader`.
            NodeRef::VariableSetStmt(set) => {
                for arg in &set.args {
                    let Some(NodeEnum::AConst(value)) = arg.node.as_ref() else {
                        continue;
                    };
                    let Some(name) = identifier_value(r, value) else {
                        continue;
                    };
                    match set.name.as_str() {
                        "search_path" => record_schema(r, &name, value.location),
                        "role" | "session_authorization" => record_role(r, &name, value.location),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    record_schema_names(r, stmt);
}

/// The name an unquoted or quoted identifier constant stands for, like `app` in
/// `set search_path to app`. `None` for string literals and numbers.
fn identifier_value(r: &Resolver, value: &AConst) -> Option<String> {
    let Some(a_const::Val::Sval(name)) = &value.val else {
        return None;
    };
    let start = usize::try_from(value.location).ok()?;
    let first = r.sql()?.as_bytes().get(start)?;
    (*first != b'\'').then(|| name.sval.clone())
}

fn record_role(r: &mut Resolver, name: &str, location: i32) {
    if let Lookup::Found(()) = r.catalog.role(name) {
        r.record(Reference::Role {
            location,
            span: r.name_span(location),
            name: name.to_owned(),
        });
    }
}

fn record_schema(r: &mut Resolver, name: &str, location: i32) {
    if let Lookup::Found(()) = r.catalog.schema(name) {
        r.record(Reference::Schema {
            location,
            span: r.name_span(location),
            name: name.to_owned(),
        });
    }
}

/// Records the schemas that a statement names after the keyword `SCHEMA` or `SCHEMAS`, like
/// `create schema app`, `drop schema if exists a, b`, `grant ... on all tables in schema app`,
/// or `alter table t set schema app`. The parse tree has no locations for them, so they are
/// found in the statement's tokens, by the names the parse tree has.
fn record_schema_names(r: &mut Resolver, stmt: &NodeEnum) {
    let names = schema_names(stmt);
    if names.is_empty() {
        return;
    }
    let Some(sql) = r.sql() else {
        return;
    };
    let Ok(scanned) = pgls_query::scan(sql) else {
        return;
    };
    let tokens: Vec<_> = scanned
        .tokens
        .iter()
        .filter(|token| !matches!(token.token(), Token::SqlComment | Token::CComment))
        .collect();
    let mut found = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(token.token(), Token::Schema | Token::Schemas) {
            continue;
        }
        let mut j = i + 1;
        while tokens
            .get(j)
            .is_some_and(|token| matches!(token.token(), Token::IfP | Token::Not | Token::Exists))
        {
            j += 1;
        }
        // A comma-separated list of names.
        while let Some(token) = tokens.get(j) {
            let Some(text) = sql.get(
                usize::try_from(token.start).unwrap_or_default()
                    ..usize::try_from(token.end).unwrap_or_default(),
            ) else {
                break;
            };
            let name = crate::identifier_name(text);
            if !names.contains(&name) {
                break;
            }
            found.push((name, token.start));
            if tokens.get(j + 1).map(|token| token.token()) != Some(Token::Ascii44) {
                break;
            }
            j += 2;
        }
    }
    for (name, location) in found {
        record_schema(r, &name, location);
    }
}

/// The schemas that a statement names on their own.
fn schema_names(stmt: &NodeEnum) -> Vec<String> {
    let strings = |nodes: &[pgls_query::Node]| -> Vec<String> {
        nodes
            .iter()
            .filter_map(|node| match node.node.as_ref()? {
                NodeEnum::String(name) => Some(name.sval.clone()),
                _ => None,
            })
            .collect()
    };
    let string = |node: Option<&pgls_query::Node>| match node.and_then(|node| node.node.as_ref()) {
        Some(NodeEnum::String(name)) => vec![name.sval.clone()],
        _ => Vec::new(),
    };
    match stmt {
        NodeEnum::CreateSchemaStmt(n) if !n.schemaname.is_empty() => vec![n.schemaname.clone()],
        NodeEnum::DropStmt(n) if n.remove_type() == ObjectType::ObjectSchema => strings(&n.objects),
        NodeEnum::GrantStmt(n)
            if n.objtype() == ObjectType::ObjectSchema
                || n.targtype() == GrantTargetType::AclTargetAllInSchema =>
        {
            strings(&n.objects)
        }
        NodeEnum::AlterDefaultPrivilegesStmt(n) => n
            .options
            .iter()
            .filter_map(|option| match option.node.as_ref()? {
                NodeEnum::DefElem(option) if option.defname == "schemas" => {
                    match option.arg.as_deref()?.node.as_ref()? {
                        NodeEnum::List(list) => Some(strings(&list.items)),
                        _ => None,
                    }
                }
                _ => None,
            })
            .flatten()
            .collect(),
        NodeEnum::AlterObjectSchemaStmt(n) => vec![n.newschema.clone()],
        NodeEnum::RenameStmt(n) if n.rename_type() == ObjectType::ObjectSchema => {
            vec![n.subname.clone()]
        }
        NodeEnum::AlterOwnerStmt(n) if n.object_type() == ObjectType::ObjectSchema => {
            string(n.object.as_deref())
        }
        NodeEnum::CommentStmt(n) if n.objtype() == ObjectType::ObjectSchema => {
            string(n.object.as_deref())
        }
        NodeEnum::CreateExtensionStmt(n) => n
            .options
            .iter()
            .filter_map(|option| match option.node.as_ref()? {
                NodeEnum::DefElem(option) if option.defname == "schema" => {
                    Some(string(option.arg.as_deref()))
                }
                _ => None,
            })
            .flatten()
            .collect(),
        _ => Vec::new(),
    }
}
