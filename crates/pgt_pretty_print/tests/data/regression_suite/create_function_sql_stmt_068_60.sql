CREATE FUNCTION functest_S_xxx(x int) RETURNS int
    LANGUAGE SQL
    AS $$ SELECT x * 2 $$
    RETURN x * 3;
