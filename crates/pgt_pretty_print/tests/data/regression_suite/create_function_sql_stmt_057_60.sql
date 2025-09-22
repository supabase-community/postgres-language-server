CREATE FUNCTION functest_S_2(a text[]) RETURNS int
    RETURN a[1]::int;
