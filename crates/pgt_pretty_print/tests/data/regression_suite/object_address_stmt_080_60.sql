WITH objects (classid, objid, objsubid) AS (VALUES
    ('pg_class'::regclass, 0, 0), -- no relation
    ('pg_class'::regclass, 'pg_class'::regclass, 100), -- no column for relation
    ('pg_proc'::regclass, 0, 0), -- no function
    ('pg_type'::regclass, 0, 0), -- no type
    ('pg_cast'::regclass, 0, 0), -- no cast
    ('pg_collation'::regclass, 0, 0), -- no collation
    ('pg_constraint'::regclass, 0, 0), -- no constraint
    ('pg_conversion'::regclass, 0, 0), -- no conversion
    ('pg_attrdef'::regclass, 0, 0), -- no default attribute
    ('pg_language'::regclass, 0, 0), -- no language
    ('pg_largeobject'::regclass, 0, 0), -- no large object, no error
    ('pg_operator'::regclass, 0, 0), -- no operator
    ('pg_opclass'::regclass, 0, 0), -- no opclass, no need to check for no access method
    ('pg_opfamily'::regclass, 0, 0), -- no opfamily
    ('pg_am'::regclass, 0, 0), -- no access method
    ('pg_amop'::regclass, 0, 0), -- no AM operator
    ('pg_amproc'::regclass, 0, 0), -- no AM proc
    ('pg_rewrite'::regclass, 0, 0), -- no rewrite
    ('pg_trigger'::regclass, 0, 0), -- no trigger
    ('pg_namespace'::regclass, 0, 0), -- no schema
    ('pg_statistic_ext'::regclass, 0, 0), -- no statistics
    ('pg_ts_parser'::regclass, 0, 0), -- no TS parser
    ('pg_ts_dict'::regclass, 0, 0), -- no TS dictionary
    ('pg_ts_template'::regclass, 0, 0), -- no TS template
    ('pg_ts_config'::regclass, 0, 0), -- no TS configuration
    ('pg_authid'::regclass, 0, 0), -- no role
    ('pg_auth_members'::regclass, 0, 0),  -- no role membership
    ('pg_database'::regclass, 0, 0), -- no database
    ('pg_tablespace'::regclass, 0, 0), -- no tablespace
    ('pg_foreign_data_wrapper'::regclass, 0, 0), -- no FDW
    ('pg_foreign_server'::regclass, 0, 0), -- no server
    ('pg_user_mapping'::regclass, 0, 0), -- no user mapping
    ('pg_default_acl'::regclass, 0, 0), -- no default ACL
    ('pg_extension'::regclass, 0, 0), -- no extension
    ('pg_event_trigger'::regclass, 0, 0), -- no event trigger
    ('pg_parameter_acl'::regclass, 0, 0), -- no parameter ACL
    ('pg_policy'::regclass, 0, 0), -- no policy
    ('pg_publication'::regclass, 0, 0), -- no publication
    ('pg_publication_namespace'::regclass, 0, 0), -- no publication namespace
    ('pg_publication_rel'::regclass, 0, 0), -- no publication relation
    ('pg_subscription'::regclass, 0, 0), -- no subscription
    ('pg_transform'::regclass, 0, 0) -- no transformation
  )
SELECT ROW(pg_identify_object(objects.classid, objects.objid, objects.objsubid))
         AS ident,
       ROW(pg_identify_object_as_address(objects.classid, objects.objid, objects.objsubid))
         AS addr,
       pg_describe_object(objects.classid, objects.objid, objects.objsubid)
         AS descr
FROM objects
ORDER BY objects.classid, objects.objid, objects.objsubid;
