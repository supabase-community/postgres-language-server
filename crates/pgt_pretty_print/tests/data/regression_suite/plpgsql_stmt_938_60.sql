CREATE OR REPLACE FUNCTION list_partitioned_table()
RETURNS SETOF public.partitioned_table.a%TYPE AS $$
DECLARE
    row public.partitioned_table%ROWTYPE;
    a_val public.partitioned_table.a%TYPE;
BEGIN
    FOR row IN SELECT * FROM public.partitioned_table ORDER BY a LOOP
        a_val := row.a;
        RETURN NEXT a_val;
    END LOOP;
    RETURN;
END; $$ LANGUAGE plpgsql;
