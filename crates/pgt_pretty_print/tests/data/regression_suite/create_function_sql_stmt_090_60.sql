CREATE FUNCTION functest_S_14() RETURNS bigint
    RETURN (SELECT count(*) FROM functestv3);
