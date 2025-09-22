CREATE OR REPLACE FUNCTION event_trigger_report_dropped()
 RETURNS event_trigger
 LANGUAGE plpgsql
AS $$
DECLARE r record;
BEGIN
    FOR r IN SELECT * from pg_event_trigger_dropped_objects()
    LOOP
    RAISE NOTICE 'DROP: orig=% normal=% istemp=% type=% identity=% schema=% name=% addr=% args=%',
        r.original, r.normal, r.is_temporary, r.object_type,
        r.object_identity, r.schema_name, r.object_name,
        r.address_names, r.address_args;
    END LOOP;
END; $$;
