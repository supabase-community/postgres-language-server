CREATE FUNCTION extstat_small(x numeric) RETURNS bool
STRICT IMMUTABLE LANGUAGE plpgsql
AS $$ BEGIN RETURN x < 1; END $$;
