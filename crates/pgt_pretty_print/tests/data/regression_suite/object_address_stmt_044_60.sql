DO $$
DECLARE
    objtype text;
    names   text[];
    args    text[];
BEGIN
    FOR objtype IN VALUES
        ('table'), ('index'), ('sequence'), ('view'),
        ('materialized view'), ('foreign table'),
        ('table column'), ('foreign table column'),
        ('aggregate'), ('function'), ('procedure'), ('type'), ('cast'),
        ('table constraint'), ('domain constraint'), ('conversion'), ('default value'),
        ('operator'), ('operator class'), ('operator family'), ('rule'), ('trigger'),
        ('text search parser'), ('text search dictionary'),
        ('text search template'), ('text search configuration'),
        ('policy'), ('user mapping'), ('default acl'), ('transform'),
        ('operator of access method'), ('function of access method'),
        ('publication namespace'), ('publication relation')
    LOOP
        FOR names IN VALUES ('{eins}'), ('{addr_nsp, zwei}'), ('{eins, zwei, drei}')
        LOOP
            FOR args IN VALUES ('{}'), ('{integer}')
            LOOP
                BEGIN
                    PERFORM pg_get_object_address(objtype, names, args);
                EXCEPTION WHEN OTHERS THEN
                    RAISE WARNING 'error for %,%,%: %', objtype, names, args, sqlerrm;
                END;
            END LOOP;
        END LOOP;
    END LOOP;
END;
$$;
