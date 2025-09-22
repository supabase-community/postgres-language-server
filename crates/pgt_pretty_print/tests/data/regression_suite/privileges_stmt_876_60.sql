CREATE OR REPLACE FUNCTION terminate_nothrow(pid int) RETURNS bool
	LANGUAGE plpgsql SECURITY DEFINER SET client_min_messages = error AS $$
BEGIN
	RETURN pg_terminate_backend($1);
EXCEPTION WHEN OTHERS THEN
	RETURN false;
END$$;
