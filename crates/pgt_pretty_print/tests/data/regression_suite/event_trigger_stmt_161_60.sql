CREATE OR REPLACE FUNCTION reindex_start_command()
RETURNS event_trigger AS $$
BEGIN
    RAISE NOTICE 'REINDEX START: % %', tg_event, tg_tag;
END;
$$ LANGUAGE plpgsql;
