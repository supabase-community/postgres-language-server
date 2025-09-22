CREATE OR REPLACE FUNCTION test_evtrig_dropped_objects() RETURNS event_trigger
LANGUAGE plpgsql AS $$
DECLARE
    obj record;
BEGIN
    FOR obj IN SELECT * FROM pg_event_trigger_dropped_objects()
    LOOP
        IF obj.object_type = 'table' THEN
                EXECUTE format('DROP TABLE IF EXISTS audit_tbls.%I',
					format('%s_%s', obj.schema_name, obj.object_name));
        END IF;

	INSERT INTO dropped_objects
		(object_type, schema_name, object_name,
		 object_identity, address_names, address_args,
		 is_temporary, original, normal) VALUES
		(obj.object_type, obj.schema_name, obj.object_name,
		 obj.object_identity, obj.address_names, obj.address_args,
		 obj.is_temporary, obj.original, obj.normal);
    END LOOP;
END
$$;
