DO $$
DECLARE
    objtype text;
BEGIN
    FOR objtype IN VALUES ('toast table'), ('index column'), ('sequence column'),
        ('toast table column'), ('view column'), ('materialized view column')
    LOOP
        BEGIN
            PERFORM pg_get_object_address(objtype, '{one}', '{}');
        EXCEPTION WHEN invalid_parameter_value THEN
            RAISE WARNING 'error for %: %', objtype, sqlerrm;
        END;
    END LOOP;
END;
$$;
