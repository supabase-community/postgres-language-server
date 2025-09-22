CREATE FUNCTION functest_IS_6()
    RETURNS int
    LANGUAGE SQL
    RETURN nextval('functest1');
