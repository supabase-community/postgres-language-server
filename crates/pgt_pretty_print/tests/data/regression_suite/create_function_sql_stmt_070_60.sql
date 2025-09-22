CREATE FUNCTION functest_S_xx(x date) RETURNS boolean
    LANGUAGE SQL
    RETURN x > 1;
