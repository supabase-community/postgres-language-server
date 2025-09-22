CREATE FUNCTION test_fdw_handler()
    RETURNS fdw_handler
    AS 'regresslib', 'test_fdw_handler'
    LANGUAGE C;
