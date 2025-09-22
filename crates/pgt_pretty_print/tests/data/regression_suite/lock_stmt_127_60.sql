CREATE FUNCTION test_atomic_ops()
    RETURNS bool
    AS 'regresslib'
    LANGUAGE C;
