CREATE FUNCTION functest_IS_7()
    RETURNS int
    LANGUAGE SQL
    RETURN (SELECT count(a) FROM functest2);
