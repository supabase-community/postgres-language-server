CREATE FUNCTION test_support_func(internal)
    RETURNS internal
    AS 'regresslib', 'test_support_func'
    LANGUAGE C STRICT;
