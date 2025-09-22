CREATE FUNCTION functest_IS_5(x int DEFAULT nextval('functest1'))
    RETURNS int
    LANGUAGE SQL
    AS 'SELECT x';
