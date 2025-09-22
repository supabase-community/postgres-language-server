CREATE FUNCTION event_trigger_dummy_trigger()
 RETURNS trigger
 LANGUAGE plpgsql
AS $$
BEGIN
    RETURN new;
END; $$;
