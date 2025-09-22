CREATE FUNCTION functest_S_1(a text, b date) RETURNS boolean
    LANGUAGE SQL
    RETURN a = 'abcd' AND b > '2001-01-01';
