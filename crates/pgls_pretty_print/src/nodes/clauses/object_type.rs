//! The keywords that name an object type in DDL: `DROP MATERIALIZED VIEW`, `COMMENT ON TEXT
//! SEARCH PARSER`, `ALTER FOREIGN DATA WRAPPER ... OWNER TO`, ...

use pgls_query::protobuf::ObjectType;

/// The keywords that name `object_type`, or `None` for a type that no statement names with
/// keywords of its own, such as the members of an operator family.
///
/// A foreign server is `SERVER`. GRANT and SECURITY LABEL name it `FOREIGN SERVER` instead.
pub(crate) fn object_type_keywords(object_type: ObjectType) -> Option<&'static str> {
    let keywords = match object_type {
        ObjectType::ObjectAccessMethod => "ACCESS METHOD",
        ObjectType::ObjectAggregate => "AGGREGATE",
        ObjectType::ObjectCast => "CAST",
        ObjectType::ObjectColumn => "COLUMN",
        ObjectType::ObjectCollation => "COLLATION",
        ObjectType::ObjectConversion => "CONVERSION",
        ObjectType::ObjectDatabase => "DATABASE",
        ObjectType::ObjectDomain => "DOMAIN",
        ObjectType::ObjectDomconstraint | ObjectType::ObjectTabconstraint => "CONSTRAINT",
        ObjectType::ObjectEventTrigger => "EVENT TRIGGER",
        ObjectType::ObjectExtension => "EXTENSION",
        ObjectType::ObjectFdw => "FOREIGN DATA WRAPPER",
        ObjectType::ObjectForeignServer => "SERVER",
        ObjectType::ObjectForeignTable => "FOREIGN TABLE",
        ObjectType::ObjectFunction => "FUNCTION",
        ObjectType::ObjectIndex => "INDEX",
        ObjectType::ObjectLanguage => "LANGUAGE",
        ObjectType::ObjectLargeobject => "LARGE OBJECT",
        ObjectType::ObjectMatview => "MATERIALIZED VIEW",
        ObjectType::ObjectOpclass => "OPERATOR CLASS",
        ObjectType::ObjectOperator => "OPERATOR",
        ObjectType::ObjectOpfamily => "OPERATOR FAMILY",
        ObjectType::ObjectPolicy => "POLICY",
        ObjectType::ObjectProcedure => "PROCEDURE",
        ObjectType::ObjectPublication => "PUBLICATION",
        ObjectType::ObjectRole => "ROLE",
        ObjectType::ObjectRoutine => "ROUTINE",
        ObjectType::ObjectRule => "RULE",
        ObjectType::ObjectSchema => "SCHEMA",
        ObjectType::ObjectSequence => "SEQUENCE",
        ObjectType::ObjectSubscription => "SUBSCRIPTION",
        ObjectType::ObjectStatisticExt => "STATISTICS",
        ObjectType::ObjectTable => "TABLE",
        ObjectType::ObjectTablespace => "TABLESPACE",
        ObjectType::ObjectTransform => "TRANSFORM",
        ObjectType::ObjectTrigger => "TRIGGER",
        ObjectType::ObjectTsconfiguration => "TEXT SEARCH CONFIGURATION",
        ObjectType::ObjectTsdictionary => "TEXT SEARCH DICTIONARY",
        ObjectType::ObjectTsparser => "TEXT SEARCH PARSER",
        ObjectType::ObjectTstemplate => "TEXT SEARCH TEMPLATE",
        ObjectType::ObjectType => "TYPE",
        ObjectType::ObjectUserMapping => "USER MAPPING",
        ObjectType::ObjectView => "VIEW",
        ObjectType::Undefined
        | ObjectType::ObjectAmop
        | ObjectType::ObjectAmproc
        | ObjectType::ObjectAttribute
        | ObjectType::ObjectDefault
        | ObjectType::ObjectDefacl
        | ObjectType::ObjectParameterAcl
        | ObjectType::ObjectPublicationNamespace
        | ObjectType::ObjectPublicationRel => return None,
    };
    Some(keywords)
}
