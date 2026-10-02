CREATE FUNCTION functest_S_13() RETURNS boolean
    BEGIN ATOMIC
        SELECT 1;
        SELECT false;
    END;
