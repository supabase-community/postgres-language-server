CREATE FUNCTION test_event_trigger_guc() RETURNS event_trigger
LANGUAGE plpgsql AS $$
DECLARE
	obj record;
BEGIN
	FOR obj IN SELECT * FROM pg_event_trigger_dropped_objects()
	LOOP
		RAISE NOTICE '% dropped %', tg_tag, obj.object_type;
	END LOOP;
END;
$$;
