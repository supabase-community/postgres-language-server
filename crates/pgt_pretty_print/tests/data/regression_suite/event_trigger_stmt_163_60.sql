CREATE FUNCTION reindex_end_command()
RETURNS event_trigger AS $$
DECLARE
    obj record;
BEGIN
    FOR obj IN SELECT * FROM pg_event_trigger_ddl_commands()
    LOOP
        RAISE NOTICE 'REINDEX END: command_tag=% type=% identity=%',
	    obj.command_tag, obj.object_type, obj.object_identity;
    END LOOP;
END;
$$ LANGUAGE plpgsql;
