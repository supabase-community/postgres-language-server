CREATE FUNCTION test_func() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
    RAISE NOTICE 'Hello, world!';
END;
$$;
