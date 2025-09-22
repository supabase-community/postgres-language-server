SELECT
    e.evtname,
    pg_describe_object('pg_event_trigger'::regclass, e.oid, 0) as descr,
    b.type, b.object_names, b.object_args,
    pg_identify_object(a.classid, a.objid, a.objsubid) as ident
  FROM pg_event_trigger as e,
    LATERAL pg_identify_object_as_address('pg_event_trigger'::regclass, e.oid, 0) as b,
    LATERAL pg_get_object_address(b.type, b.object_names, b.object_args) as a
  ORDER BY e.evtname;
