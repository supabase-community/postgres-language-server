CREATE FUNCTION unwanted_grant_nofail(int) RETURNS int
	IMMUTABLE LANGUAGE plpgsql AS $$
BEGIN
	PERFORM public.unwanted_grant();
	RAISE WARNING 'owned';
	RETURN 1;
EXCEPTION WHEN OTHERS THEN
	RETURN 2;
END$$;
