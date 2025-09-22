CREATE FUNCTION functest_S_xx(x anyarray) RETURNS anyelement
    LANGUAGE SQL
    RETURN x[1];
